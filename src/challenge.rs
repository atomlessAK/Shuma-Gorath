// src/challenge.rs
// Interactive math challenge for banned users

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

pub trait KeyValueStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()>;
    fn delete(&self, key: &str) -> Result<(), ()>;
}

impl KeyValueStore for Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        Store::get(self, key).map_err(|_| ())
    }
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        Store::set(self, key, value).map_err(|_| ())
    }
    fn delete(&self, key: &str) -> Result<(), ()> {
        Store::delete(self, key).map_err(|_| ())
    }
}

use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use percent_encoding;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChallengeSeed {
    pub seed_id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub ip_bucket: String,
    pub grid_size: u8,
    pub active_cells: u8,
    pub transforms: Vec<Transform>,
    pub training_count: u8,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Transform {
    RotateCw90,
    RotateCcw90,
    MirrorHorizontal,
    MirrorVertical,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    DropTop,
    DropBottom,
    DropLeft,
    DropRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChallengePuzzle {
    pub training_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    pub test_input: Vec<u8>,
    pub test_output: Vec<u8>,
    pub grid_size: usize,
}

fn get_challenge_secret() -> String {
    std::env::var("CHALLENGE_SECRET")
        .or_else(|_| std::env::var("API_KEY"))
        .unwrap_or_else(|_| "changeme-challenge-secret".to_string())
}

fn sign_payload(payload: &str) -> Vec<u8> {
    let secret = get_challenge_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_signature(payload: &str, sig: &[u8]) -> bool {
    let secret = get_challenge_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.verify_slice(sig).is_ok()
}

pub(crate) fn make_seed_token(payload: &ChallengeSeed) -> String {
    let payload_json = serde_json::to_string(payload).unwrap();
    let sig = sign_payload(&payload_json);
    let payload_b64 = general_purpose::STANDARD.encode(payload_json.as_bytes());
    let sig_b64 = general_purpose::STANDARD.encode(sig);
    format!("{}.{}", payload_b64, sig_b64)
}

pub(crate) fn parse_seed_token(token: &str) -> Result<ChallengeSeed, &'static str> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or("missing payload")?;
    let sig_b64 = parts.next().ok_or("missing signature")?;
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64.as_bytes())
        .map_err(|_| "invalid payload")?;
    let sig = general_purpose::STANDARD
        .decode(sig_b64.as_bytes())
        .map_err(|_| "invalid signature")?;
    let payload_json = String::from_utf8(payload_bytes).map_err(|_| "invalid payload")?;

    if !verify_signature(&payload_json, &sig) {
        return Err("signature mismatch");
    }

    serde_json::from_str::<ChallengeSeed>(&payload_json).map_err(|_| "invalid payload")
}

pub(crate) fn build_puzzle(seed: &ChallengeSeed) -> ChallengePuzzle {
    let size = seed.grid_size as usize;
    let active = seed.active_cells as usize;
    let mut rng = StdRng::seed_from_u64(seed.seed);
    let mut training_pairs = Vec::new();
    for _ in 0..seed.training_count {
        let (input, output) = generate_pair(&mut rng, size, active, &seed.transforms);
        training_pairs.push((input, output));
    }
    let (test_input, test_output) = generate_pair(&mut rng, size, active, &seed.transforms);
    ChallengePuzzle {
        training_pairs,
        test_input,
        test_output,
        grid_size: size,
    }
}

fn generate_grid(rng: &mut impl Rng, size: usize, active: usize) -> Vec<u8> {
    let mut grid = vec![0u8; size * size];
    let mut indices: Vec<usize> = (0..grid.len()).collect();
    indices.shuffle(rng);
    let active_indices: Vec<usize> = indices.into_iter().take(active).collect();
    let mut has_one = false;
    let mut has_two = false;
    for idx in &active_indices {
        let val = if rng.random::<bool>() { 1 } else { 2 };
        if val == 1 {
            has_one = true;
        } else {
            has_two = true;
        }
        grid[*idx] = val;
    }
    if active >= 2 && (!has_one || !has_two) {
        let idx = active_indices[0];
        grid[idx] = if has_one { 2 } else { 1 };
    }
    grid
}

fn is_inverse_rotation_pair(a: Transform, b: Transform) -> bool {
    matches!(
        (a, b),
        (Transform::RotateCw90, Transform::RotateCcw90) | (Transform::RotateCcw90, Transform::RotateCw90)
    )
}

fn all_transforms() -> Vec<Transform> {
    vec![
        Transform::RotateCw90,
        Transform::RotateCcw90,
        Transform::MirrorHorizontal,
        Transform::MirrorVertical,
        Transform::ShiftUp,
        Transform::ShiftDown,
        Transform::ShiftLeft,
        Transform::ShiftRight,
        Transform::DropTop,
        Transform::DropBottom,
        Transform::DropLeft,
        Transform::DropRight,
    ]
}

pub(crate) fn select_transform_pair(rng: &mut impl Rng) -> Vec<Transform> {
    let mut options = all_transforms();
    loop {
        options.shuffle(rng);
        let a = options[0];
        let b = options[1];
        if !is_inverse_rotation_pair(a, b) {
            return vec![a, b];
        }
    }
}

fn apply_transforms(grid: &[u8], size: usize, transforms: &[Transform]) -> Vec<u8> {
    let mut current = grid.to_vec();
    for t in transforms {
        current = apply_transform(&current, size, *t);
    }
    current
}

pub(crate) fn generate_pair(
    rng: &mut impl Rng,
    size: usize,
    active: usize,
    transforms: &[Transform],
) -> (Vec<u8>, Vec<u8>) {
    const MAX_PAIR_ATTEMPTS: usize = 64;
    let mut last_input = Vec::new();
    let mut last_output = Vec::new();
    for _ in 0..MAX_PAIR_ATTEMPTS {
        let input = generate_grid(rng, size, active);
        let output = apply_transforms(&input, size, transforms);
        if output != input {
            return (input, output);
        }
        last_input = input;
        last_output = output;
    }
    (last_input, last_output)
}

pub(crate) fn apply_transform(grid: &[u8], size: usize, transform: Transform) -> Vec<u8> {
    let mut out = vec![0u8; size * size];
    match transform {
        Transform::RotateCw90 => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(c, size - 1 - r, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::RotateCcw90 => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(size - 1 - c, r, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::MirrorHorizontal => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r, size - 1 - c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::MirrorVertical => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(size - 1 - r, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftUp | Transform::DropTop => {
            for r in 1..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r - 1, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftDown | Transform::DropBottom => {
            for r in 0..size - 1 {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r + 1, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftLeft | Transform::DropLeft => {
            for r in 0..size {
                for c in 1..size {
                    let src = idx(r, c, size);
                    let dst = idx(r, c - 1, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftRight | Transform::DropRight => {
            for r in 0..size {
                for c in 0..size - 1 {
                    let src = idx(r, c, size);
                    let dst = idx(r, c + 1, size);
                    out[dst] = grid[src];
                }
            }
        }
    }
    out
}

pub(crate) fn parse_submission(input: &str, size: usize) -> Result<Vec<u8>, &'static str> {
    let trimmed = input.trim();
    let expected = size * size;
    if trimmed.is_empty() {
        return Err("invalid length");
    }
    if !trimmed.chars().all(|c| c == '0' || c == '1' || c == '2') {
        return Err("invalid format");
    }
    if trimmed.len() != expected {
        return Err("invalid length");
    }
    let out = trimmed
        .chars()
        .map(|c| match c {
            '0' => 0,
            '1' => 1,
            _ => 2,
        })
        .collect();
    Ok(out)
}

fn grid_to_tritstring(grid: &[u8]) -> String {
    grid.iter().map(|v| char::from(b'0' + *v)).collect()
}

fn render_grid(grid: &[u8], size: usize, class_name: &str, clickable: bool) -> String {
    let mut html = String::new();
    html.push_str(&format!(
        "<div class=\"grid {}\" style=\"grid-template-columns: repeat({}, 28px);\">",
        class_name,
        size
    ));
    for (idx, val) in grid.iter().enumerate() {
        let mut classes = String::from("cell");
        if *val == 1 {
            classes.push_str(" active");
        } else if *val == 2 {
            classes.push_str(" active-alt");
        }
        if clickable {
            classes.push_str(" clickable");
        }
        html.push_str(&format!("<div class=\"{}\" data-idx=\"{}\"></div>", classes, idx));
    }
    html.push_str("</div>");
    html
}

fn idx(row: usize, col: usize, size: usize) -> usize {
    row * size + col
}

pub(crate) fn render_challenge(req: &Request) -> Response {
    let ip = crate::extract_client_ip(req);
    let ip_bucket = crate::ip::bucket_ip(&ip);
    let now = crate::admin::now_ts();
    let mut rng = rand::rng();
    let grid_size = 4u8;
    let active_cells = rng.random_range(5..=7);
    let transforms = select_transform_pair(&mut rng);
    let seed = ChallengeSeed {
        seed_id: format!("{:016x}", rng.random::<u64>()),
        issued_at: now,
        expires_at: now + 300,
        ip_bucket,
        grid_size,
        active_cells,
        transforms,
        training_count: 2,
        seed: rng.random::<u64>(),
    };
    let puzzle = build_puzzle(&seed);
    let seed_token = make_seed_token(&seed);
    let output_size = grid_size as usize * grid_size as usize;
    let empty_output = vec![0u8; output_size];

    let training_html: String = puzzle
        .training_pairs
        .iter()
        .enumerate()
        .map(|(idx, (input, output))| {
            format!(
                "<div class=\"pair\"><div class=\"pair-title\">Example {}</div>{}{}{}</div>",
                idx + 1,
                "<div class=\"pair-grids\">",
                format!(
                    "<div><div class=\"grid-label\">Input</div>{}</div><div><div class=\"grid-label\">Output</div>{}</div>",
                    render_grid(input, puzzle.grid_size, "grid-static", false),
                    render_grid(output, puzzle.grid_size, "grid-static", false),
                ),
                "</div>"
            )
        })
        .collect();

    let legend_transforms = all_transforms();
    let legend_html = render_transform_legend(&legend_transforms);
    let html = format!(r#"
        <html>
        <head>
          <style>
            body {{ font-family: sans-serif; background: #f7f7f7; margin: 24px; color: #111; }}
            .challenge {{ max-width: 900px; margin: 0 auto; background: #fff; padding: 24px; border: 1px solid #e5e7eb; }}
            :root {{ --cell-on: #111; --cell-alt: rgb(255,205,235); }}
            .grid {{ display: grid; grid-template-columns: repeat(4, 28px); gap: 4px; }}
            .cell {{ width: 28px; height: 28px; border: 1px solid #ddd; background: #fff; }}
            .cell.active {{ background: var(--cell-on); }}
            .cell.active-alt {{ background: var(--cell-alt); }}
            .cell.clickable {{ cursor: pointer; }}
            .pair {{ margin-bottom: 16px; }}
            .pair-title {{ font-weight: 600; margin-bottom: 8px; }}
            .pair-grids {{ display: flex; gap: 24px; align-items: flex-start; }}
            .grid-label {{ font-size: 12px; color: #6b7280; margin-bottom: 6px; }}
            .test-block {{ margin-top: 20px; padding-top: 16px; border-top: 1px solid #eee; }}
            .test-grids {{ display: inline-grid; grid-template-columns: auto auto; gap: 24px; align-items: start; }}
            .submit-row {{ grid-column: 1 / -1; margin-top: 12px; }}
            .submit-row button {{ width: 100%; }}
            .grid-output .cell {{ cursor: pointer; }}
            .grid-output .cell.clickable {{ cursor: pointer; }}
            button {{ padding: 8px 14px; font-size: 14px; }}
            .debug-panel {{ margin-top: 12px; padding: 10px 12px; border: 1px dashed #cbd5f5; background: #f8fafc; font-size: 12px; color: #334155; }}
            .legend {{ margin: 12px 0 16px; padding: 12px; border: 1px solid #e5e7eb; background: #f8fafc; }}
            .legend-title {{ font-weight: 600; margin-bottom: 6px; }}
            .legend-subtitle {{ font-size: 12px; color: #6b7280; margin-bottom: 10px; }}
            .legend-items {{ display: flex; flex-wrap: wrap; gap: 12px; }}
            .legend-item {{ display: flex; flex-direction: column; align-items: center; gap: 6px; background: #fff; border: 1px solid #e5e7eb; padding: 8px; min-width: 84px; }}
            .legend-icon {{ position: relative; width: 64px; height: 64px; flex: 0 0 auto; }}
            .legend-grid {{ position: absolute; inset: 0; display: grid; grid-template-columns: repeat(4, 1fr); gap: 2px; }}
            .legend-cell {{ border: 1px solid #e2e8f0; background: #fff; }}
            .legend-cell.on {{ background: var(--cell-on); }}
            .legend-cell.alt {{ background: var(--cell-alt); }}
            .legend-line {{ position: absolute; border-top: 1px dashed rgb(255,205,235); left: 0; right: 0; }}
            .legend-line.vert {{ border-top: 0; border-left: 1px dashed rgb(255,205,235); top: 0; bottom: 0; left: 50%; }}
            .legend-line.line-h-0 {{ top: 0%; }}
            .legend-line.line-h-25 {{ top: 25%; }}
            .legend-line.line-h-50 {{ top: 50%; }}
            .legend-line.line-h-75 {{ top: 75%; }}
            .legend-line.line-h-100 {{ top: 100%; }}
            .legend-line.line-v-0 {{ left: 0%; }}
            .legend-line.line-v-25 {{ left: 25%; }}
            .legend-line.line-v-50 {{ left: 50%; }}
            .legend-line.line-v-75 {{ left: 75%; }}
            .legend-line.line-v-100 {{ left: 100%; }}
            .legend-arrow {{ position: absolute; color: #111; font-size: 16px; line-height: 1; }}
            .legend-arrow.arrow-center {{ top: 50%; left: 50%; transform: translate(-50%, -50%); }}
            .legend-arrow.arrow-up {{ top: 6px; left: 50%; transform: translateX(-50%); }}
            .legend-arrow.arrow-down {{ bottom: 6px; left: 50%; transform: translateX(-50%); }}
            .legend-arrow.arrow-left {{ left: 6px; top: 50%; transform: translateY(-50%); }}
            .legend-arrow.arrow-right {{ right: 6px; top: 50%; transform: translateY(-50%); }}
            .legend-label {{ font-size: 11px; color: #111; text-transform: lowercase; }}
          </style>
        </head>
        <body>
          <div class="challenge">
            <h2>Human Verification Challenge</h2>
            <p>Infer the rule from the examples, then complete the output grid.</p>
            {legend_html}
            {training_html}
            <div class="test-block">
              <div class="pair-title">Your turn</div>
              <div class="test-grids">
                <div>
                  <div class="grid-label">Input</div>
                  {test_input}
                </div>
                <div>
                  <div class="grid-label">Output</div>
                  <div id="challenge-output-grid">
                    {test_output}
                  </div>
                </div>
                <form method="POST" action="/challenge" class="submit-row">
                  <input type="hidden" name="seed" value="{seed_token}" />
                  <input type="hidden" name="output" id="challenge-output" value="{empty_tritstring}" />
                  <button type="submit">Submit</button>
                </form>
              </div>
              <div class="debug-panel">
                Debug transforms: {debug_transforms}
              </div>
            </div>
          </div>
          <script>
            const size = {grid_size};
            const output = Array(size * size).fill(0);
            const outputField = document.getElementById('challenge-output');
            function updateOutput() {{
              outputField.value = output.join('');
            }}
            function applyCellState(cell, value) {{
              cell.classList.remove('active', 'active-alt');
              if (value === 1) {{
                cell.classList.add('active');
              }} else if (value === 2) {{
                cell.classList.add('active-alt');
              }}
            }}
            updateOutput();
            const outputGrid = document.getElementById('challenge-output-grid');
            outputGrid.addEventListener('click', (event) => {{
              const cell = event.target.closest('.cell');
              if (!cell || !cell.dataset.idx) {{
                return;
              }}
              const idx = parseInt(cell.dataset.idx, 10);
              output[idx] = (output[idx] + 1) % 3;
              applyCellState(cell, output[idx]);
              updateOutput();
            }});
          </script>
        </body>
        </html>
    "#,
    legend_html = legend_html,
    training_html = training_html,
    test_input = render_grid(&puzzle.test_input, puzzle.grid_size, "grid-static", false),
    test_output = render_grid(&empty_output, puzzle.grid_size, "grid-output", true),
    seed_token = seed_token,
    grid_size = grid_size,
    empty_tritstring = grid_to_tritstring(&empty_output),
    debug_transforms = seed.transforms.iter().map(transform_label).collect::<Vec<_>>().join(" then "),
    );
    Response::new(200, html)
}

fn transform_label(transform: &Transform) -> &'static str {
    match transform {
        Transform::RotateCw90 => "rotate 90 deg clockwise",
        Transform::RotateCcw90 => "rotate 90 deg counter-clockwise",
        Transform::MirrorHorizontal => "mirror horizontally",
        Transform::MirrorVertical => "mirror vertically",
        Transform::ShiftUp => "shift up 1 row",
        Transform::ShiftDown => "shift down 1 row",
        Transform::ShiftLeft => "shift left 1 column",
        Transform::ShiftRight => "shift right 1 column",
        Transform::DropTop => "drop top row",
        Transform::DropBottom => "drop bottom row",
        Transform::DropLeft => "drop left column",
        Transform::DropRight => "drop right column",
    }
}

fn render_transform_legend(transforms: &[Transform]) -> String {
    let items: String = transforms
        .iter()
        .map(|transform| {
            let label = transform_legend_label(transform);
            let icon = render_transform_icon(transform);
            format!(
                "<div class=\"legend-item\"><div class=\"legend-label\">{}</div>{}</div>",
                label, icon
            )
        })
        .collect();
    format!(
        "<div class=\"legend\"><div class=\"legend-title\">Available transforms</div><div class=\"legend-subtitle\">Two of these are being applied to the input grids.</div><div class=\"legend-items\">{}</div></div>",
        items
    )
}

fn transform_legend_label(transform: &Transform) -> &'static str {
    match transform {
        Transform::RotateCw90 | Transform::RotateCcw90 => "90°",
        Transform::MirrorHorizontal | Transform::MirrorVertical => "mirror",
        Transform::ShiftUp | Transform::ShiftDown | Transform::ShiftLeft | Transform::ShiftRight => "shift",
        Transform::DropTop | Transform::DropBottom | Transform::DropLeft | Transform::DropRight => "drop",
    }
}

fn render_transform_icon(transform: &Transform) -> String {
    let mut overlays = String::new();
    match transform {
        Transform::RotateCw90 => overlays.push_str("<div class=\"legend-arrow arrow-center\">&#x21bb;</div>"),
        Transform::RotateCcw90 => overlays.push_str("<div class=\"legend-arrow arrow-center\">&#x21ba;</div>"),
        Transform::MirrorHorizontal => overlays.push_str("<div class=\"legend-line line-h-50\"></div>"),
        Transform::MirrorVertical => overlays.push_str("<div class=\"legend-line vert line-v-50\"></div>"),
        Transform::ShiftUp => overlays.push_str("<div class=\"legend-line line-h-25\"></div><div class=\"legend-arrow arrow-up\">&#x2191;</div>"),
        Transform::ShiftDown => overlays.push_str("<div class=\"legend-line line-h-75\"></div><div class=\"legend-arrow arrow-down\">&#x2193;</div>"),
        Transform::ShiftLeft => overlays.push_str("<div class=\"legend-line vert line-v-25\"></div><div class=\"legend-arrow arrow-left\">&#x2190;</div>"),
        Transform::ShiftRight => overlays.push_str("<div class=\"legend-line vert line-v-75\"></div><div class=\"legend-arrow arrow-right\">&#x2192;</div>"),
        Transform::DropTop => overlays.push_str("<div class=\"legend-line line-h-0\"></div><div class=\"legend-arrow arrow-up\">&#x2191;</div>"),
        Transform::DropBottom => overlays.push_str("<div class=\"legend-line line-h-100\"></div><div class=\"legend-arrow arrow-down\">&#x2193;</div>"),
        Transform::DropLeft => overlays.push_str("<div class=\"legend-line vert line-v-0\"></div><div class=\"legend-arrow arrow-left\">&#x2190;</div>"),
        Transform::DropRight => overlays.push_str("<div class=\"legend-line vert line-v-100\"></div><div class=\"legend-arrow arrow-right\">&#x2192;</div>"),
    }
    let grid = render_legend_grid();
    format!("<div class=\"legend-icon\">{}{}</div>", grid, overlays)
}

fn render_legend_grid() -> String {
    let mut html = String::from("<div class=\"legend-grid\">");
    let pattern = [
        1u8, 2u8, 0u8, 0u8,
        1u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8,
    ];
    for val in pattern.iter() {
        let mut classes = "legend-cell".to_string();
        if *val == 1 {
            classes.push_str(" on");
        } else if *val == 2 {
            classes.push_str(" alt");
        }
        html.push_str(&format!("<div class=\"{}\"></div>", classes));
    }
    html.push_str("</div>");
    html
}

pub(crate) fn serve_challenge_page(req: &Request, test_mode: bool) -> Response {
    if !test_mode {
        return Response::new(404, "Not Found");
    }
    render_challenge(req)
}

pub fn handle_challenge_submit<S: KeyValueStore>(store: &S, req: &Request) -> Response {
    let form = String::from_utf8_lossy(req.body()).to_string();
    let seed_token = match get_form_field(&form, "seed") {
        Some(v) => v,
        None => return Response::new(400, "Missing seed"),
    };
    let output_raw = match get_form_field(&form, "output") {
        Some(v) => v,
        None => return Response::new(400, "Missing output"),
    };
    let seed = match parse_seed_token(&seed_token) {
        Ok(s) => s,
        Err(_) => return Response::new(400, "Invalid seed"),
    };
    let now = crate::admin::now_ts();
    if now > seed.expires_at {
        return Response::new(400, "Seed expired");
    }
    let ip = crate::extract_client_ip(req);
    let ip_bucket = crate::ip::bucket_ip(&ip);
    if seed.ip_bucket != ip_bucket {
        return Response::new(400, "IP bucket mismatch");
    }
    let used_key = format!("challenge_used:{}", seed.seed_id);
    if let Ok(Some(val)) = store.get(&used_key) {
        if let Ok(stored) = String::from_utf8(val) {
            if let Ok(exp) = stored.parse::<u64>() {
                if now <= exp {
                    return Response::new(400, "Seed already used");
                }
            }
        }
        let _ = store.delete(&used_key);
    }
    let output = match parse_submission(&output_raw, seed.grid_size as usize) {
        Ok(v) => v,
        Err(e) => return Response::new(400, e),
    };
    let puzzle = build_puzzle(&seed);
    if output == puzzle.test_output {
        let _ = store.set(&used_key, seed.expires_at.to_string().as_bytes());
        return Response::new(200, "<html><body><h2>Thank you! Challenge complete.</h2></body></html>");
    }
    Response::new(403, "<html><body><h2 style='color:red;'>Incorrect. Try again.</h2><a href='/challenge'>Back to challenge</a></body></html>")
}

fn get_form_field(form: &str, name: &str) -> Option<String> {
    for pair in form.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == name {
                return Some(url_decode(v));
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s).decode_utf8_lossy().to_string()
}

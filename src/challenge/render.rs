use rand::Rng;
use spin_sdk::http::{Request, Response};

use super::puzzle::{build_puzzle, select_transform_pair, transforms_for_count};
use super::token::make_seed_token;
use super::types::{ChallengeSeed, Transform};
use super::{challenge_response, PUZZLE_PATH};

pub(crate) fn render_challenge(req: &Request, transform_count: usize) -> Response {
    let ip = crate::extract_client_ip(req);
    let ip_bucket = crate::signals::ip::bucket_ip(&ip);
    let now = crate::admin::now_ts();
    let mut rng = rand::rng();
    let grid_size = 4u8;
    let active_cells = rng.random_range(7..=9);
    let legend_transforms = transforms_for_count(transform_count);
    let transforms = select_transform_pair(&mut rng, &legend_transforms);
    let seed = ChallengeSeed {
        seed_id: format!("{:016x}", rng.random::<u64>()),
        issued_at: now,
        expires_at: now + 300,
        ip_bucket,
        grid_size,
        active_cells,
        transforms,
        training_count: 1,
        seed: rng.random::<u64>(),
    };
    let puzzle = build_puzzle(&seed);
    let seed_token = make_seed_token(&seed);
    let output_size = grid_size as usize * grid_size as usize;
    let empty_output = vec![0u8; output_size];
    let test_input_json = serde_json::to_string(&puzzle.test_input).unwrap();

    let training_html: String = puzzle
        .training_pairs
        .iter()
        .map(|(input, output)| {
            format!(
                "<div class=\"pair\">{}{}{}</div>",
                "<div class=\"pair-grids\">",
                format!(
                    "<div><div class=\"grid-label\">Before</div>{}</div><div><div class=\"grid-label\">After</div>{}</div>",
                    render_grid(input, puzzle.grid_size, "grid-static", false),
                    render_grid(output, puzzle.grid_size, "grid-static", false),
                ),
                "</div>"
            )
        })
        .collect();

    let legend_html = render_transform_legend(&legend_transforms);
    let html = format!(
        r#"
        <html>
        <head>
          <style>
            :root {{
              --color-black: #111;
              --color-pink: rgb(255,205,235);
              --color-green: rgb(105, 205, 135);
              --color-dark-gray: #475569;
              --color-border: #e2e8f0;
              --color-legend-bg: #f8fafc;
              --color-white: #fff;
              --font-body: 1rem;
              --font-small: clamp(0.9rem, 0.86rem + 0.25vw, 0.96rem);
              --font-medium: clamp(1.2rem, 1rem + 0.25vw, 1.5rem);
              --font-heading: clamp(1.7rem, 1.45rem + 0.9vw, 2.1rem);
              --puzzle-cell: clamp(30px, 5vw, 36px);
              --puzzle-gap: 4px;
              --puzzle-grid-size: calc((var(--puzzle-cell) * 4) + (var(--puzzle-gap) * 3));
              --duo-grid-gap: clamp(12px, 2vw, 24px);
              --duo-grid-size: calc((var(--puzzle-grid-size) * 2) + var(--duo-grid-gap));
              --legend-gap: 2px;
            }}
            body {{ font-family: sans-serif; font-size: 15px; line-height: 1.6; background: repeating-linear-gradient(-45deg,#fffafd,#fffafd 10px,#faf2fa 10px,#faf2fa 20px); margin: 24px; color: var(--color-black); }}
            .challenge {{ max-width: 980px; margin: 0 auto; background: var(--color-white); padding: 24px; border: 1px solid var(--color-border); }}
            .challenge h2 {{ width: var(--duo-grid-size); margin: 0 auto 0.6rem; font-size: var(--font-heading); line-height: 1.2; text-align: center; }}
            .grid {{ display: grid; gap: var(--puzzle-gap); }}
            .cell {{ width: var(--puzzle-cell); height: var(--puzzle-cell); border: 1px solid var(--color-border); background: var(--color-white); }}
            .cell.active {{ background: var(--color-black); }}
            .cell.active-alt {{ background: var(--color-pink); }}
            .pair {{ margin-bottom: 16px; }}
            .pair-grids {{ display: grid; grid-template-columns: repeat(2, var(--puzzle-grid-size)); gap: var(--duo-grid-gap); align-items: flex-start; justify-content: center; width: var(--duo-grid-size); margin: 0 auto; }}
            .grid-label {{ font-size: var(--font-small); color: var(--color-dark-gray); margin-bottom: 6px; }}
            .test-block {{ margin-top: 14px; }}
            .test-grids {{ display: grid; grid-template-columns: repeat(2, var(--puzzle-grid-size)); gap: var(--duo-grid-gap); align-items: start; justify-content: center; width: var(--duo-grid-size); margin: 0 auto; }}
            .submit-row {{ grid-column: 1 / -1; margin-top: 12px; }}
            .submit-row button {{ width: 100%; }}
            button {{ padding: 8px 14px; font-size: var(--font-body); background: var(--color-black); color: var(--color-white); border: 1px solid var(--color-black); }}
            .legend {{ margin: 12px 0 16px; padding: 12px; border: 1px solid var(--color-border); background: var(--color-legend-bg); }}
            .legend-fieldset {{ border: 0; margin: 0; padding: 0; min-width: 0; }}
            .legend-subtitle {{ font-size: var(--font-medium); color: var(--color-black); margin: 0 auto 10px; width: var(--duo-grid-size); text-align: center; }}
            .legend-options {{ width: var(--duo-grid-size); margin: 0 auto; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }}
            .legend-row {{ display: flex; flex-direction: column; align-items: center; justify-content: flex-start; gap: 6px; border: 1px solid transparent; padding: 0; }}
            .legend-row.is-selected {{ border-color: var(--color-black); }}
            .legend-item {{ display: flex; flex-direction: column; align-items: center; gap: 0; min-width: 0; width: 100%; }}
            .legend-picks {{ display: flex; align-items: center; gap: 10px; }}
            .legend-pick-label {{ display: inline-flex; align-items: center; gap: 4px; font-size: var(--font-small); color: var(--color-dark-gray); cursor: pointer; }}
            .legend-pick-label input {{ width: 16px; height: 16px; margin: 0; accent-color: var(--color-black); cursor: pointer; }}
            .legend-icon {{ position: relative; width: 100%; max-width: 100%; aspect-ratio: 1 / 1; flex: 0 0 auto; }}
            .legend-grid {{ position: absolute; inset: 0; display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--legend-gap); z-index: 0; }}
            .legend-cell {{ border: 1px solid var(--color-border); background: var(--color-white); }}
            .legend-line {{ position: absolute; border-top: 2px dashed var(--color-pink); left: 0; right: 0; z-index: 1; }}
            .legend-line.vert {{ border-top: 0; border-left: 2px dashed var(--color-pink); top: 0; bottom: 0; left: 50%; }}
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
            .legend-arrow {{ position: absolute; color: var(--color-green); font-size: 2.4rem; line-height: 1; font-weight: normal; z-index: 1; }}
            .legend-arrow.arrow-center {{ top: 65%; left: 50%; transform: translate(-50%, -50%); }}
            .legend-arrow.arrow-up {{ top: 0; left: 50%; transform: translateX(-50%); }}
            .legend-arrow.arrow-down {{ bottom: 0; left: 50%; transform: translateX(-50%); }}
            .legend-arrow.arrow-left {{ left: 0; top: 50%; transform: translateY(-50%); }}
            .legend-arrow.arrow-right {{ right: 0; top: 50%; transform: translateY(-50%); }}
            .legend-label {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 100%; text-align: center; font-size: var(--font-small); color: var(--color-dark-gray); text-transform: capitalize; line-height: 1; z-index: 2; pointer-events: none; }}
            @media (max-width: 640px) {{
              .legend-options {{ grid-template-columns: repeat(2, minmax(0, 1fr)); }}
            }}
            @media (max-width: 480px) {{
              body {{ margin: 12px; }}
              .challenge {{ padding: 16px; }}
              :root {{
                --puzzle-cell: clamp(26px, 8vw, 32px);
                --duo-grid-gap: 12px;
              }}
            }}
            @media (max-width: 400px) {{
              .challenge h2 {{ width: var(--puzzle-grid-size); }}
              .legend-subtitle, .legend-options {{ width: var(--puzzle-grid-size); }}
              .legend-options {{ grid-template-columns: repeat(2, minmax(0, 1fr)); }}
              .pair-grids, .test-grids {{ grid-template-columns: 1fr; width: var(--puzzle-grid-size); gap: 12px; }}
              .submit-row {{ grid-column: 1; }}
            }}
          </style>
        </head>
        <body>
          <div class="challenge">
            <h2>Puzzle</h2>
            {training_html}
            <div class="legend">
              {legend_html}
              <div class="test-block">
                <div class="test-grids">
                  <div>
                    {test_input}
                  </div>
                  <div>
                    <div id="challenge-output-grid">
                      {test_output}
                    </div>
                  </div>
                  <form method="POST" action="{puzzle_path}" class="submit-row">
                    <input type="hidden" name="seed" value="{seed_token}" />
                    <input type="hidden" name="output" id="challenge-output" value="{empty_tritstring}" />
                    <button type="submit">Submit</button>
                  </form>
                </div>
              </div>
            </div>
          </div>
          <script>
            const size = {grid_size};
            const baseInput = {test_input_json};
            const output = Array(size * size).fill(0);
            const outputField = document.getElementById('challenge-output');
            const outputCells = Array.from(document.querySelectorAll('#challenge-output-grid .cell'));
            const transform1Radios = Array.from(document.querySelectorAll('input[name="transform_1"]'));
            const transform2Radios = Array.from(document.querySelectorAll('input[name="transform_2"]'));
            const legendRows = Array.from(document.querySelectorAll('.legend-row'));
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
            function selectedValue(radios) {{
              const current = radios.find((radio) => radio.checked);
              return current ? current.value : null;
            }}
            function selectedTransforms() {{
              return [selectedValue(transform1Radios), selectedValue(transform2Radios)].filter(Boolean);
            }}
            function syncLegendState() {{
              const first = selectedValue(transform1Radios);
              const second = selectedValue(transform2Radios);
              for (const row of legendRows) {{
                const value = row.dataset.transform;
                const selected = value === first || value === second;
                row.classList.toggle('is-selected', selected);
              }}
            }}
            function applyTransform(grid, transform) {{
              const out = Array(size * size).fill(0);
              const idx = (r, c) => (r * size) + c;
              if (transform === 'rotate_cw90') {{
                for (let r = 0; r < size; r++) {{
                  for (let c = 0; c < size; c++) {{
                    out[idx(c, size - 1 - r)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'rotate_ccw90') {{
                for (let r = 0; r < size; r++) {{
                  for (let c = 0; c < size; c++) {{
                    out[idx(size - 1 - c, r)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'mirror_horizontal') {{
                for (let r = 0; r < size; r++) {{
                  for (let c = 0; c < size; c++) {{
                    out[idx(size - 1 - r, c)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'mirror_vertical') {{
                for (let r = 0; r < size; r++) {{
                  for (let c = 0; c < size; c++) {{
                    out[idx(r, size - 1 - c)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'shift_up') {{
                for (let r = 1; r < size; r++) {{
                  for (let c = 0; c < size; c++) {{
                    out[idx(r - 1, c)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'shift_down') {{
                for (let r = 0; r < size - 1; r++) {{
                  for (let c = 0; c < size; c++) {{
                    out[idx(r + 1, c)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'shift_left') {{
                for (let r = 0; r < size; r++) {{
                  for (let c = 1; c < size; c++) {{
                    out[idx(r, c - 1)] = grid[idx(r, c)];
                  }}
                }}
              }} else if (transform === 'shift_right') {{
                for (let r = 0; r < size; r++) {{
                  for (let c = 0; c < size - 1; c++) {{
                    out[idx(r, c + 1)] = grid[idx(r, c)];
                  }}
                }}
              }}
              return out;
            }}
            function renderOutput() {{
              const selected = selectedTransforms();
              let current = selected.length ? baseInput.slice() : Array(size * size).fill(0);
              for (const t of selected) {{
                current = applyTransform(current, t);
              }}
              for (let i = 0; i < current.length; i++) {{
                output[i] = current[i];
                applyCellState(outputCells[i], output[i]);
              }}
              updateOutput();
            }}
            function onLegendChange(event) {{
              if (event.target.name === 'transform_1') {{
                const conflict = transform2Radios.find((radio) => radio.checked && radio.value === event.target.value);
                if (conflict) {{
                  conflict.checked = false;
                }}
              }} else if (event.target.name === 'transform_2') {{
                const conflict = transform1Radios.find((radio) => radio.checked && radio.value === event.target.value);
                if (conflict) {{
                  conflict.checked = false;
                }}
              }}
              syncLegendState();
              renderOutput();
            }}
            updateOutput();
            transform1Radios.forEach((radio) => radio.addEventListener('change', onLegendChange));
            transform2Radios.forEach((radio) => radio.addEventListener('change', onLegendChange));
            syncLegendState();
            renderOutput();
          </script>
        </body>
        </html>
    "#,
        legend_html = legend_html,
        training_html = training_html,
        test_input = render_grid(&puzzle.test_input, puzzle.grid_size, "grid-static", false),
        test_output = render_grid(&empty_output, puzzle.grid_size, "grid-output", false),
        test_input_json = test_input_json,
        seed_token = seed_token,
        grid_size = grid_size,
        empty_tritstring = grid_to_tritstring(&empty_output),
        puzzle_path = PUZZLE_PATH,
    );
    challenge_response(200, &html)
}

fn transform_value(transform: Transform) -> &'static str {
    match transform {
        Transform::RotateCw90 => "rotate_cw90",
        Transform::RotateCcw90 => "rotate_ccw90",
        Transform::MirrorHorizontal => "mirror_horizontal",
        Transform::MirrorVertical => "mirror_vertical",
        Transform::ShiftUp => "shift_up",
        Transform::ShiftDown => "shift_down",
        Transform::ShiftLeft => "shift_left",
        Transform::ShiftRight => "shift_right",
        Transform::DropTop => "drop_top",
        Transform::DropBottom => "drop_bottom",
        Transform::DropLeft => "drop_left",
        Transform::DropRight => "drop_right",
    }
}

fn transform_option_label(transform: Transform) -> &'static str {
    match transform {
        Transform::ShiftUp => "Shift up",
        Transform::ShiftDown => "Shift down",
        Transform::ShiftLeft => "Shift left",
        Transform::ShiftRight => "Shift right",
        Transform::RotateCw90 => "90&#176; clockwise",
        Transform::RotateCcw90 => "90&#176; anticlockwise",
        Transform::MirrorHorizontal => "Mirror horizontally",
        Transform::MirrorVertical => "Mirror vertically",
        Transform::DropTop => "Drop top",
        Transform::DropBottom => "Drop bottom",
        Transform::DropLeft => "Drop left",
        Transform::DropRight => "Drop right",
    }
}

fn render_transform_legend(transforms: &[Transform]) -> String {
    let rows: String = transforms
        .iter()
        .map(|transform| {
            let label = transform_legend_label(transform);
            let icon = render_transform_icon(transform, label);
            let value = transform_value(*transform);
            let aria_label = transform_option_label(*transform);
            format!(
                "<div class=\"legend-row\" data-transform=\"{}\"><div class=\"legend-item\">{}</div><div class=\"legend-picks\"><label class=\"legend-pick-label\"><input type=\"radio\" class=\"legend-radio legend-radio-1\" name=\"transform_1\" value=\"{}\" aria-label=\"First transform: {}\" /><span>1st</span></label><label class=\"legend-pick-label\"><input type=\"radio\" class=\"legend-radio legend-radio-2\" name=\"transform_2\" value=\"{}\" aria-label=\"Second transform: {}\" /><span>2nd</span></label></div></div>",
                value, icon, value, aria_label, value, aria_label
            )
        })
        .collect();
    format!(
        "<fieldset class=\"legend-fieldset\"><div class=\"legend-subtitle\">Which 2 transforms were applied?</div><div class=\"legend-options\">{}</div></fieldset>",
        rows
    )
}

fn transform_legend_label(transform: &Transform) -> &'static str {
    match transform {
        Transform::RotateCw90 | Transform::RotateCcw90 => "90&#176;",
        Transform::MirrorHorizontal | Transform::MirrorVertical => "mirror",
        Transform::ShiftUp
        | Transform::ShiftDown
        | Transform::ShiftLeft
        | Transform::ShiftRight => "shift",
        Transform::DropTop | Transform::DropBottom | Transform::DropLeft | Transform::DropRight => {
            "shift"
        }
    }
}

fn render_transform_icon(transform: &Transform, label: &str) -> String {
    let mut overlays = String::new();
    match transform {
        Transform::RotateCw90 => {
            overlays.push_str("<div class=\"legend-arrow arrow-center\">&#x21bb;</div>")
        }
        Transform::RotateCcw90 => {
            overlays.push_str("<div class=\"legend-arrow arrow-center\">&#x21ba;</div>")
        }
        Transform::MirrorHorizontal => overlays.push_str("<div class=\"legend-line line-h-50\"></div>"),
        Transform::MirrorVertical => {
            overlays.push_str("<div class=\"legend-line vert line-v-50\"></div>")
        }
        Transform::ShiftUp => overlays.push_str(
            "<div class=\"legend-line line-h-25\"></div><div class=\"legend-arrow arrow-up\">&#x2191;</div>",
        ),
        Transform::ShiftDown => overlays.push_str(
            "<div class=\"legend-line line-h-75\"></div><div class=\"legend-arrow arrow-down\">&#x2193;</div>",
        ),
        Transform::ShiftLeft => overlays.push_str(
            "<div class=\"legend-line vert line-v-25\"></div><div class=\"legend-arrow arrow-left\">&#x2190;</div>",
        ),
        Transform::ShiftRight => overlays.push_str(
            "<div class=\"legend-line vert line-v-75\"></div><div class=\"legend-arrow arrow-right\">&#x2192;</div>",
        ),
        Transform::DropTop => overlays.push_str(
            "<div class=\"legend-line line-h-0\"></div><div class=\"legend-arrow arrow-up\">&#x2191;</div>",
        ),
        Transform::DropBottom => overlays.push_str(
            "<div class=\"legend-line line-h-100\"></div><div class=\"legend-arrow arrow-down\">&#x2193;</div>",
        ),
        Transform::DropLeft => overlays.push_str(
            "<div class=\"legend-line vert line-v-0\"></div><div class=\"legend-arrow arrow-left\">&#x2190;</div>",
        ),
        Transform::DropRight => overlays.push_str(
            "<div class=\"legend-line vert line-v-100\"></div><div class=\"legend-arrow arrow-right\">&#x2192;</div>",
        ),
    }
    let grid = render_legend_grid();
    format!(
        "<div class=\"legend-icon\">{}{}<div class=\"legend-label\">{}</div></div>",
        grid, overlays, label
    )
}

fn render_legend_grid() -> String {
    let mut html = String::from("<div class=\"legend-grid\">");
    for _ in 0..16 {
        html.push_str("<div class=\"legend-cell\"></div>");
    }
    html.push_str("</div>");
    html
}

fn grid_to_tritstring(grid: &[u8]) -> String {
    grid.iter().map(|v| char::from(b'0' + *v)).collect()
}

fn render_grid(grid: &[u8], size: usize, class_name: &str, clickable: bool) -> String {
    let mut html = String::new();
    html.push_str(&format!(
        "<div class=\"grid {}\" style=\"grid-template-columns: repeat({}, var(--puzzle-cell));\">",
        class_name, size
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
        html.push_str(&format!(
            "<div class=\"{}\" data-idx=\"{}\"></div>",
            classes, idx
        ));
    }
    html.push_str("</div>");
    html
}

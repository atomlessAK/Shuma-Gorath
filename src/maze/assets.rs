use spin_sdk::http::{Method, Response};

pub(crate) const MAZE_STYLE_PATH: &str = "/maze/assets/maze.4be8d1c.min.css";
pub(crate) const MAZE_SCRIPT_PATH: &str = "/maze/assets/maze.2f1c84d.min.js";
pub(crate) const MAZE_WORKER_PATH: &str = "/maze/assets/maze-worker.a2d6c13.min.js";

const MAZE_STYLE_CSS: &str = "body{margin:0;padding:24px;background:radial-gradient(circle at 15% 15%,#0b1020 0,#020617 70%);color:#111827;font-family:\"IBM Plex Sans\",\"Segoe UI\",system-ui,sans-serif}body.style-lite{background:#0b1020}a{color:inherit}.wrap{max-width:1120px;margin:0 auto;background:#fff;border:1px solid #e5e7eb;border-radius:14px;overflow:hidden;box-shadow:0 24px 54px rgba(2,6,23,.3)}header{padding:20px 26px;background:#0f172a;color:#e2e8f0}body.style-lite header{padding:14px 18px}.crumb{margin-top:6px;opacity:.82;font-size:.88rem}.content{padding:24px;background:#f8fafc}.description{background:#fff;border-left:4px solid #38bdf8;border-radius:8px;padding:12px;line-height:1.65;margin:0 0 12px}.nav-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(224px,1fr));gap:12px;margin-top:14px}.nav-card{text-decoration:none;display:block;background:#fff;border:1px solid #e5e7eb;border-radius:10px;padding:14px;transition:transform .15s ease,border-color .15s ease,box-shadow .15s ease}.nav-card:hover{transform:translateY(-2px);border-color:#38bdf8;box-shadow:0 10px 18px rgba(15,23,42,.12)}.nav-card h3{margin:0 0 6px;font-size:.95rem;color:#0f172a}.nav-card p{margin:0;color:#475569;font-size:.84rem;line-height:1.45}.arrow{margin-top:8px;color:#2563eb;font-size:.82rem}.pow-hint{margin-top:6px;font-size:.74rem;color:#7c2d12;background:#ffedd5;border-radius:999px;display:inline-block;padding:2px 8px}.hidden-link{position:absolute!important;width:1px;height:1px;margin:-1px;padding:0;border:0;clip:rect(0 0 0 0);clip-path:inset(50%);overflow:hidden;white-space:nowrap}body.style-lite .description{padding:8px 10px;margin-bottom:8px;border-left-width:3px}body.style-lite .nav-grid{gap:8px}body.style-lite .nav-card{padding:10px}body.style-lite .arrow{display:none}";

const MAZE_SCRIPT_JS: &str = "(function(){const bEl=document.getElementById('maze-bootstrap');const nav=document.getElementById('maze-nav-grid');if(!bEl||!nav)return;let b={};try{b=JSON.parse(bEl.textContent||'{}')}catch(_e){return}const assets=b.assets||{};const exp=b.client_expansion||{};const lowMem=Number(navigator.deviceMemory||0)>0&&Number(navigator.deviceMemory)<=2;const lowCores=Number(navigator.hardwareConcurrency||0)>0&&Number(navigator.hardwareConcurrency)<=2;const saveData=!!(navigator.connection&&navigator.connection.saveData);const constrained=lowMem||lowCores||saveData;const signedHiddenCount=Math.max(0,Number(exp.hidden_count)||0);const requestedHiddenCount=Math.max(0,Math.min(signedHiddenCount,constrained?2:signedHiddenCount));const powMaxIterations=constrained?220000:600000;let worker=null;let powSeq=0;const powPending={};function ensureWorker(){if(worker||typeof Worker==='undefined')return worker;const src=assets.worker_url||'/maze/assets/maze-worker.a2d6c13.min.js';try{worker=new Worker(src)}catch(_e){worker=null;return null}worker.onmessage=async function(ev){const d=(ev&&ev.data)||{};if(d.kind==='pow_result'&&d.id){const done=powPending[d.id];if(done){delete powPending[d.id];done(d.nonce||null)}return}if(d.kind==='candidates'){const c=Array.isArray(d.candidates)?d.candidates:[];const links=await issueHidden(c);appendHidden(links)}};return worker}function solvePowWithWorker(token,difficulty){const w=ensureWorker();if(!w)return Promise.resolve(null);const id=String(++powSeq);return new Promise((resolve)=>{powPending[id]=resolve;w.postMessage({type:'pow',id:id,token:token,difficulty:difficulty,max_iterations:powMaxIterations});setTimeout(()=>{if(powPending[id]){delete powPending[id];resolve(null)}},15000)})}function attachPow(a){const raw=a.getAttribute('data-pow-difficulty');if(!raw)return;const d=parseInt(raw,10);if(!Number.isFinite(d)||d<=0)return;a.addEventListener('click',async function(ev){if(a.dataset.powReady==='1')return;ev.preventDefault();a.dataset.powReady='0';const href=new URL(a.href,window.location.origin);const t=href.searchParams.get('mt')||'';const nonce=await solvePowWithWorker(t,d);if(nonce!==null){href.searchParams.set('mpn',String(nonce));a.dataset.powReady='1';window.location.assign(href.toString());return}window.location.assign(href.toString())})}async function sendCheckpoint(){if(!b.checkpoint_token)return;try{await fetch('/maze/checkpoint',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({token:b.checkpoint_token,flow_id:b.flow_id,depth:b.depth,checkpoint_reason:'page_load'}),keepalive:true})}catch(_e){}}async function issueHidden(c){if(!Array.isArray(c)||c.length===0)return[];if(!b.checkpoint_token||!exp.seed_sig)return[];const issuePath=exp.issue_path||'/maze/issue-links';try{const resp=await fetch(issuePath,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({parent_token:b.checkpoint_token||'',flow_id:b.flow_id||'',entropy_nonce:b.entropy_nonce||'',path_prefix:b.path_prefix||'/maze/',seed:exp.seed||0,seed_sig:exp.seed_sig||'',hidden_count:signedHiddenCount,requested_hidden_count:requestedHiddenCount,segment_len:exp.segment_len||16,candidates:c})});if(!resp.ok)return[];const data=await resp.json();return Array.isArray(data.links)?data.links:[]}catch(_e){return[]}}function appendHidden(links){for(const h of links){const a=document.createElement('a');a.href=h.href;a.className='hidden-link';a.textContent=h.text||'detail';if(h.pow_difficulty)a.setAttribute('data-pow-difficulty',String(h.pow_difficulty));nav.appendChild(a);attachPow(a)}}function spawnHiddenGeneration(){if(!exp.enabled||!requestedHiddenCount)return;const w=ensureWorker();if(!w)return;w.postMessage({type:'generate',seed:exp.seed||0,hidden_count:requestedHiddenCount,path_prefix:b.path_prefix||'/maze/',segment_len:exp.segment_len||16})}const anchors=nav.querySelectorAll('a[data-pow-difficulty]');anchors.forEach(attachPow);sendCheckpoint();spawnHiddenGeneration()})();";

const MAZE_WORKER_JS: &str = "function nextSeed(seed){seed^=seed<<13;seed^=seed>>>7;seed^=seed<<17;return Math.abs(seed>>>0)}function leadingZeroBitsOk(bytes,bits){let r=bits;for(let i=0;i<bytes.length;i+=1){if(r<=0)return true;const v=bytes[i];if(r>=8){if(v!==0)return false;r-=8}else{const m=0xff<<(8-r);return (v&m)===0}}return true}self.onmessage=async function(ev){const d=ev&&ev.data?ev.data:{};if(d.type==='pow'){const token=String(d.token||'');const difficulty=Math.max(1,Math.min(24,Number(d.difficulty)||1));const maxIter=Math.max(1,Math.min(800000,Number(d.max_iterations)||600000));for(let nonce=0;nonce<maxIter;nonce+=1){const raw=new TextEncoder().encode(token+':'+nonce);const hash=await crypto.subtle.digest('SHA-256',raw);const bytes=new Uint8Array(hash);if(leadingZeroBitsOk(bytes,difficulty)){self.postMessage({kind:'pow_result',id:String(d.id||''),nonce:String(nonce)});return}}self.postMessage({kind:'pow_result',id:String(d.id||''),nonce:null});return}let seed=Number(d.seed)||0;const count=Math.max(0,Math.min(24,Number(d.hidden_count)||0));const segLen=Math.max(8,Math.min(48,Number(d.segment_len)||16));const prefix=(typeof d.path_prefix==='string'&&d.path_prefix.startsWith('/'))?d.path_prefix:'/maze/';const nouns=['index','ledger','stream','matrix','archive','catalog','window','route','segment','cache'];const verbs=['sync','render','compose','index','align','trace','map','verify','queue','persist'];const areas=['ops','network','session','storage','policy','gateway','coordination'];function seg(){const chars='abcdefghijklmnopqrstuvwxyz0123456789';let out='';for(let i=0;i<segLen;i+=1){seed=nextSeed(seed);out+=chars[seed%chars.length]}return out}const candidates=[];for(let i=0;i<count;i+=1){const path=prefix+seg();const base=(seed+i)>>>0;const text=(verbs[base%verbs.length]+' '+nouns[(base>>>3)%nouns.length]+' '+areas[(base>>>6)%areas.length]);const description='Operational '+nouns[(base>>>9)%nouns.length]+' '+areas[(base>>>12)%areas.length]+' '+verbs[(base>>>15)%verbs.length]+'.';candidates.push({path:path,text:text,description:description})}self.postMessage({kind:'candidates',candidates:candidates})};";

fn static_asset_response(content_type: &str, body: &str, include_body: bool) -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(if include_body { body } else { "" })
        .build()
}

pub(crate) fn maybe_handle_asset(path: &str, method: &Method) -> Option<Response> {
    if !matches!(method, Method::Get | Method::Head) {
        return None;
    }
    let include_body = *method != Method::Head;
    match path {
        MAZE_STYLE_PATH => Some(static_asset_response(
            "text/css; charset=utf-8",
            MAZE_STYLE_CSS,
            include_body,
        )),
        MAZE_SCRIPT_PATH => Some(static_asset_response(
            "application/javascript; charset=utf-8",
            MAZE_SCRIPT_JS,
            include_body,
        )),
        MAZE_WORKER_PATH => Some(static_asset_response(
            "application/javascript; charset=utf-8",
            MAZE_WORKER_JS,
            include_body,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{maybe_handle_asset, MAZE_SCRIPT_PATH, MAZE_STYLE_PATH, MAZE_WORKER_PATH};
    use spin_sdk::http::Method;

    #[test]
    fn versioned_maze_assets_are_served_with_immutable_cache() {
        let script = maybe_handle_asset(MAZE_SCRIPT_PATH, &Method::Get)
            .expect("script asset should be served");
        let cache_control = script
            .headers()
            .find(|(name, _)| name.eq_ignore_ascii_case("cache-control"))
            .and_then(|(_, value)| value.as_str())
            .unwrap_or_default();
        assert_eq!(cache_control, "public, max-age=31536000, immutable");

        assert!(maybe_handle_asset(MAZE_STYLE_PATH, &Method::Get).is_some());
        assert!(maybe_handle_asset(MAZE_WORKER_PATH, &Method::Get).is_some());
    }

    #[test]
    fn head_asset_requests_are_header_only() {
        let response = maybe_handle_asset(MAZE_SCRIPT_PATH, &Method::Head)
            .expect("head request should be served");
        assert!(response.body().is_empty());
    }
}

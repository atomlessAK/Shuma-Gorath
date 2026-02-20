// src/block_page.rs
// Customizable HTML block pages for WASM Bot Defence

pub enum BlockReason {
    Honeypot,
    RateLimit,
    OutdatedBrowser,
    GeoPolicy,
    IpRangePolicy,
}

pub fn render_block_page(reason: BlockReason) -> String {
    match reason {
        BlockReason::Honeypot => BLOCK_HONEYPOT_HTML.to_string(),
        BlockReason::RateLimit => BLOCK_RATELIMIT_HTML.to_string(),
        BlockReason::OutdatedBrowser => BLOCK_BROWSER_HTML.to_string(),
        BlockReason::GeoPolicy => BLOCK_GEO_HTML.to_string(),
        BlockReason::IpRangePolicy => BLOCK_IP_RANGE_HTML.to_string(),
    }
}

const BLOCK_HONEYPOT_HTML: &str = r#"
<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"UTF-8\">
  <title>Access Blocked</title>
  <style>
    body { font-family: sans-serif; background: #f9f9f9; margin: 2em; }
    .block-container { background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 480px; margin: auto; }
    h1 { color: #c00; }
  </style>
</head>
<body>
  <div class=\"block-container\">
    <h1>Access Blocked</h1>
    <p>Your request triggered a security honeypot and has been blocked for your protection.</p>
    <p>If you believe this is an error, please contact the site administrator.</p>
  </div>
</body>
</html>
"#;

const BLOCK_RATELIMIT_HTML: &str = r#"
<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"UTF-8\">
  <title>Rate Limit Exceeded</title>
  <style>
    body { font-family: sans-serif; background: #f9f9f9; margin: 2em; }
    .block-container { background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 480px; margin: auto; }
    h1 { color: #c60; }
  </style>
</head>
<body>
  <div class=\"block-container\">
    <h1>Rate Limit Exceeded</h1>
    <p>Too many requests have been received from your IP address. Please try again later.</p>
    <p>If you believe this is an error, contact the site administrator.</p>
  </div>
</body>
</html>
"#;

const BLOCK_BROWSER_HTML: &str = r#"
<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"UTF-8\">
  <title>Browser Not Supported</title>
  <style>
    body { font-family: sans-serif; background: #f9f9f9; margin: 2em; }
    .block-container { background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 480px; margin: auto; }
    h1 { color: #c00; }
  </style>
</head>
<body>
  <div class=\"block-container\">
    <h1>Browser Not Supported</h1>
    <p>Your browser version is not supported for security reasons. Please update your browser and try again.</p>
    <p>If you believe this is an error, contact the site administrator.</p>
  </div>
</body>
</html>
"#;

const BLOCK_GEO_HTML: &str = r#"
<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"UTF-8\">
  <title>Access Restricted</title>
  <style>
    body { font-family: sans-serif; background: #f9f9f9; margin: 2em; }
    .block-container { background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 480px; margin: auto; }
    h1 { color: #c00; }
  </style>
</head>
<body>
  <div class=\"block-container\">
    <h1>Access Restricted</h1>
    <p>Your request was blocked by regional access policy.</p>
    <p>If you believe this is an error, contact the site administrator.</p>
  </div>
</body>
</html>
"#;

const BLOCK_IP_RANGE_HTML: &str = r#"
<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"UTF-8\">
  <title>Access Restricted</title>
  <style>
    body { font-family: sans-serif; background: #f9f9f9; margin: 2em; }
    .block-container { background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 480px; margin: auto; }
    h1 { color: #c00; }
  </style>
</head>
<body>
  <div class=\"block-container\">
    <h1>Access Restricted</h1>
    <p>Your request was blocked by network policy.</p>
    <p>If you believe this is an error, contact the site administrator.</p>
  </div>
</body>
</html>
"#;

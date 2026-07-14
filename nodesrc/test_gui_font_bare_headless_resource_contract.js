const fs = require("fs");
function read(file) { return fs.readFileSync(file, "utf8").replace(/\r\n/g, "\n"); }
function assert(value, message) { if (!value) throw new Error(message); }
const bare = read("stdlib/platforms/gui/bare/font_resource_provider.nepl");
const headless = read("stdlib/platforms/gui/headless/font_resource_provider.nepl");
for (const name of ["open", "byte_len", "read_bytes", "close"]) assert(bare.includes(`"font_resource_${name}"`), `missing ${name}`);
assert(bare.includes("GuiFontResourceSource::EmbeddedBlob"), "Bare source mismatch");
assert(bare.includes("gui_font_resource_validate_request_bytes_hash"), "Bare hash missing");
assert(headless.includes("str_eq requested_text fixture_path"), "Headless exact path missing");
assert(headless.includes("UnsupportedDecodePolicy") && headless.includes("PayloadNotBinary"), "Headless validation missing");
assert(headless.includes("gui_font_resource_validate_request_bytes_hash"), "Headless hash missing");
for (const source of [bare, headless]) assert(!/(?:std\/fs|fontconfig|DirectWrite|CoreText|MockTextMeasurer)/i.test(source), "forbidden fallback");
console.log("GUI font Bare/Headless resource provider contract passed");

-- Nekomusume candidate dissector; experimental/non-frozen research aid only.
local p_neko = Proto("nekomusume_candidate", "Nekomusume Candidate (experimental)")
local f_magic = ProtoField.string("neko.magic", "Magic")
local f_version = ProtoField.uint8("neko.version", "Version", base.DEC)
local f_type = ProtoField.uint8("neko.frame_type", "Frame type", base.HEX)
local f_len = ProtoField.uint16("neko.frame_length", "Frame length", base.DEC)
p_neko.fields = {f_magic, f_version, f_type, f_len}
function p_neko.dissector(buf, pkt, tree)
  if buf:len() < 9 or buf(0,2):string() ~= "NK" then return 0 end
  local root = tree:add(p_neko, buf, "Nekomusume candidate (non-frozen)")
  root:add(f_magic, buf(0,2)); root:add(f_version, buf(2,1))
  local off = 9
  while off + 3 <= buf:len() do
    local t = buf(off,1); local n = buf(off+1,2):uint()
    local frame = root:add(p_neko, buf(off, math.min(3+n, buf:len()-off)), "SessionRecord frame")
    frame:add(f_type, t); frame:add(f_len, buf(off+1,2))
    if off + 3 + n > buf:len() then break end
    off = off + 3 + n
  end
  return buf:len()
end
DissectorTable.get("udp.port"):add(40080, p_neko)
DissectorTable.get("tcp.port"):add(40080, p_neko)

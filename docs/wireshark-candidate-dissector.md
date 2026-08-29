# Experimental Wireshark candidate dissector

`tools/wireshark/nekomusume_candidate.lua` is a non-authoritative visualization
helper for the current candidate NK header and Frame payload grammar. It is not
frozen, does not decrypt or authenticate payloads, and is not an interoperability
or security test. It registers only port `40080` and should be loaded manually
in an isolated research profile.

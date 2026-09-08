# Superseded candidate observation — exact `c81fa73`

> Reviewer EPREB-006/007 identified a missing authenticated promotion synchronization barrier and semantic misuse of an unsolicited D064 `ReadinessResponse` candidate sentinel. This record is retained for provenance but is **not accepted release evidence** and must not be used to claim endpoint-rebinding closure. A fresh observation is required after the barrier repair and exact-head gates.

The original bounded run used the exact `c81fa73` release binary and completed a same-Session source-port change with endpoint A → B, authenticated challenge/response, generation-1 promotion, and post-promotion DeliveryAck. It changed no route, firewall, DNS, proxy, tunnel, qdisc, namespace, or production service and cleaned all temporary artifacts. Raw endpoints, credentials, keys, and diagnostics remain intentionally omitted.

The original event and resource details are retained only as historical candidate provenance; they do not establish a valid WAN/source-endpoint claim under the corrected ordering/semantic contract.
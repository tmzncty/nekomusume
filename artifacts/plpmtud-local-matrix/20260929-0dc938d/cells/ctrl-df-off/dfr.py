import socket
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(('0.0.0.0', 40099)); s.settimeout(1.5)
try:
    d, _ = s.recvfrom(9000); print('recv', len(d))
except socket.timeout:
    print('timeout')

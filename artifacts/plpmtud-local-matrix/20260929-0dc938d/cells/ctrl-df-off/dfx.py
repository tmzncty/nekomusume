import socket, sys, errno
fam = socket.AF_INET6 if sys.argv[1] == '6' else socket.AF_INET
dst, size, mode = sys.argv[2], int(sys.argv[3]), sys.argv[4]
s = socket.socket(fam, socket.SOCK_DGRAM)
if mode == 'probe':
    opt = 23 if fam == socket.AF_INET6 else 10
    s.setsockopt(socket.IPPROTO_IPV6 if fam == socket.AF_INET6 else socket.IPPROTO_IP, opt, 3)
try:
    s.sendto(b'x' * size, (dst, 40099)); print('sent')
except OSError as e:
    print('ERR', errno.errorcode.get(e.errno, e.errno))

import serial
import hashlib

fd = serial.Serial("/dev/ttyUSB0")
fd.baudrate = 115_200

m = hashlib.sha256()
m.update(b"swap wen?")
digest = m.digest()

fd.write(bytes([1]))
fd.write(digest)

e = fd.read(1)
print(e)


fd.close()
# low_ram.py
import time

# Reservar 1 MB de RAM
size = 1024 * 1024
buffer = bytearray(size)

# Llenar el buffer
for i in range(size):
    buffer[i] = i % 256

# Mantener el programa corriendo
time.sleep(300)

# high_ram.py
import time

# Reservar 1 GB de RAM
size = 1024 * 1024 * 1024
buffer = bytearray(size)

# Llenar el buffer para asegurarse de que la memoria está realmente asignada
for i in range(size):
    buffer[i] = i % 256

# Mantener el programa corriendo
time.sleep(300)

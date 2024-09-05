# Alto consumo de RAM
docker build -t high_ram_image -f ./High-cpu/dockerfile .

# Alto consumo de CPU
docker build -t high_cpu_image -f ./High-ram/dockerfile .

# Bajo consumo de RAM
docker build -t low_ram_image -f ./Low-cpu/dockerfile .

# Bajo consumo de CPU
docker build -t low_cpu_image -f ./Low-ram/dockerfile .

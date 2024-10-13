#!/bin/bash

# Verificar y construir las imágenes solo si no existen
if [ -z "$(docker images -q high-cpu-image)" ]; then
    docker build -t high-cpu-image ./High-cpu/
fi

if [ -z "$(docker images -q low-cpu-image)" ]; then
    docker build -t low-cpu-image ./Low-cpu/
fi

if [ -z "$(docker images -q high-ram-image)" ]; then
    docker build -t high-ram-image ./High-ram/
fi

if [ -z "$(docker images -q low-ram-image)" ]; then
    docker build -t low-ram-image ./Low-ram/
fi

# Eliminar todos los contenedores existentes excepto el contenedor de Docker Compose
if [ "$(docker ps -a -q)" ]; then
    # Lista todos los contenedores excepto el contenedor 'log_container'
    containers_to_remove=$(docker ps -a -q | grep -v $(docker ps -a -q -f name=log_container))
    if [ -n "$containers_to_remove" ]; then
        docker rm -f $containers_to_remove
    fi
fi

# Array de imágenes
images=("high-ram-image" "high-cpu-image" "low-ram-image" "low-cpu-image")

# Crear 10 contenedores aleatorios
for i in {1..10}
do
    # Seleccionar una imagen aleatoriamente
    image=${images[$RANDOM % ${#images[@]}]}

    # Generar un nombre aleatorio para el contenedor
    container_name=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 13)

    # Ejecutar el contenedor
    docker run -d --name $container_name $image
done

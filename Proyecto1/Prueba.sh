# cd ./Docker/High-ram
# cd ./Docker/High-cpu
# cd ./Docker/Low-ram
cd ./Docker/Low-cpu
# echo "Construyendo imagen..."
# gcc high-cpu.c -o high-cpu
# ./high-cpu


gcc low-cpu.c -o low-cpu
./low-cpu
#!/bin/bash

# Ruta al archivo de cronjobs temporales
TEMP_CRON="/tmp/mycron"

# Obtener los cronjobs existentes y guardarlos en un archivo temporal
crontab -l > $TEMP_CRON

# Agregar el nuevo cronjob (reemplaza la ruta del script y el intervalo según sea necesario)
echo "* * * * * /home/jefferson/Escritorio/lab-sopes1/Proyecto1/Docker/Cronjob.sh" >> $TEMP_CRON
echo "* * * * * sleep 30; /home/jefferson/Escritorio/lab-sopes1/Proyecto1/Docker/Cronjob.sh" >> $TEMP_CRON

# Aplicar el archivo temporal al crontab
crontab $TEMP_CRON

# Limpiar el archivo temporal
rm $TEMP_CRON

echo "Cronjob añadido con éxito."

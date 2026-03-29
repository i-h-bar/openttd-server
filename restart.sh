#!/bin/bash

docker stop openttd-server
echo "Stopped"

docker start openttd-server
echo "Started"

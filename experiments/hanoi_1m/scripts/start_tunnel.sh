#!/bin/bash\n# Sets up SSH tunnel to Windows GPU node\nssh -L 8080:127.0.0.1:8080 windows1-w1 -N -f\necho "Tunnel established to windows1-w1:8080"

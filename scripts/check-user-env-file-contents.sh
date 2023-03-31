#!/usr/bin/env bash

STATUS=0

RED='\033[0;31m'
BOLD='\033[1;37m'
NC='\033[0m' # No Color

check_required_var () {
  if [ -z "${!1}" ]; then
    printf "${RED}error:${NC} Please add the required ${BOLD}$1${NC} variable to your local .env file. "
    echo $2
    STATUS=1
  fi
}

exit $STATUS

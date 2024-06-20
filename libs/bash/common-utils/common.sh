#!/usr/bin/env bash

look_for_ptau_file() {
  local phase1_file="$1"

  if [ -f "${phase1_file}" ]; then
    echo "Found Phase 1 ptau file"
  else
    echo "No Phase 1 ptau file found. Exiting..."
    exit 1
  fi
}

create_build_folder() {
  local build_dir="$1"

  if [ ! -d "${build_dir}" ]; then
    echo "No build directory found. Creating build directory..."
    mkdir -p "${build_dir}"
  fi
}

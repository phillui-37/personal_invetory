die() {
  printf '%s\n' "error: $*" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
usage:
  bin/app start backend
  bin/app start frontend <macos|windows|linux|ios|android>
  bin/app build backend
  bin/app build frontend <macos|windows|linux|ios|android>
  bin/app test <backend|frontend|all>
  bin/app clean
  bin/app gen-api
EOF
  exit 1
}

host_platform() {
  case "$(uname -s)" in
    Darwin) printf 'macos\n' ;;
    Linux) printf 'linux\n' ;;
    MINGW*|MSYS*|CYGWIN*) printf 'windows\n' ;;
    *) printf 'unknown\n' ;;
  esac
}

require_frontend_target() {
  target=$1
  host=$(host_platform)

  case "$target" in
    macos)
      [ "$host" = "macos" ] || die "frontend target '$target' requires a macOS host"
      ;;
    windows)
      [ "$host" = "windows" ] || die "frontend target '$target' requires a Windows host"
      ;;
    linux)
      [ "$host" = "linux" ] || die "frontend target '$target' requires a Linux host"
      ;;
    ios)
      [ "$host" = "macos" ] || die "frontend target '$target' requires a macOS host"
      ;;
    android)
      case "$host" in
        macos|linux|windows) ;;
        *) die "frontend target '$target' is unsupported on host '$host'" ;;
      esac
      ;;
    *)
      die "unsupported frontend target '$target'"
      ;;
  esac

  [ -d "$REPO_ROOT/frontend/$target" ] || die "frontend target '$target' is not configured in this repo"
}

copy_artifact() {
  src=$1
  dest=$2

  [ -e "$src" ] || die "missing artifact '$src'"
  mkdir -p "$(dirname "$dest")"
  rm -rf "$dest"

  if [ -d "$src" ]; then
    cp -R "$src" "$dest"
  else
    cp "$src" "$dest"
  fi
}

find_single_path() {
  search_dir=$1
  pattern=$2

  [ -d "$search_dir" ] || die "missing search directory '$search_dir'"

  set -- "$search_dir"/$pattern
  [ -e "$1" ] || die "no artifacts matching '$pattern' under '$search_dir'"
  [ $# -eq 1 ] || die "multiple artifacts matching '$pattern' under '$search_dir'"
  printf '%s\n' "$1"
}

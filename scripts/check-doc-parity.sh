#!/usr/bin/env sh
set -eu

root="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"
english="$root/docs/user/en"
korean="$root/docs/user/ko"

english_files="$(find "$english" -type f -printf '%P\n' | sort)"
korean_files="$(find "$korean" -type f -printf '%P\n' | sort)"

if [ "$english_files" != "$korean_files" ]; then
    echo "English and Korean user-document paths differ." >&2
    printf 'English:\n%s\nKorean:\n%s\n' "$english_files" "$korean_files" >&2
    exit 1
fi

echo "User-document path parity passed."

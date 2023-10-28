#!/bin/sh

set -e
corepack enable

echo "[[ Installing npm packages ]]"
pnpm i

echo "[[ Building ]]"
pnpm build

echo "[[ Starting server ]]"
pnpm start

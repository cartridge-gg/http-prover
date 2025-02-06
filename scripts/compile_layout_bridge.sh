set -eux

CAIRO_COMPILE="cairo-lang/src/starkware/cairo/lang/scripts/cairo-compile"

python "$CAIRO_COMPILE" \
    cairo-lang/src/starkware/cairo/cairo_verifier/layouts/all_cairo/cairo_verifier.cairo \
    --output layout_bridge.json 
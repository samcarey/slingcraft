run:
    BEVY_ASSET_ROOT="." dx serve --hot-patch

run-web:
    trunk serve & echo "http://127.0.0.1:8080/index.html#dev"
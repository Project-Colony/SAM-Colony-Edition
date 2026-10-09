# Changelog

## [0.7.1](https://github.com/Project-Colony/SAM-Colony-Edition/compare/v0.7.0...v0.7.1) (2026-10-09)


### Fixes

* **bootstrap:** extract the Steam runtime into a directory per version ([#13](https://github.com/Project-Colony/SAM-Colony-Edition/issues/13)) ([b32f57c](https://github.com/Project-Colony/SAM-Colony-Edition/commit/b32f57c5a33cfb03e03dc0ffb29217eacb4ffc92))
* **deps:** update serde_with to 3.24.0 ([#9](https://github.com/Project-Colony/SAM-Colony-Edition/issues/9)) ([be6e3bc](https://github.com/Project-Colony/SAM-Colony-Edition/commit/be6e3bc9d80be87ada9e13a0af6fff4f3c223cef))
* **deps:** update steamworks to 0.13.1 ([#10](https://github.com/Project-Colony/SAM-Colony-Edition/issues/10)) ([75cb408](https://github.com/Project-Colony/SAM-Colony-Edition/commit/75cb408155deca6a41238e26b0416c8c129eaeb0))
* **deps:** update vite, postcss, devalue and source-map-js ([#8](https://github.com/Project-Colony/SAM-Colony-Edition/issues/8)) ([ca6e8d4](https://github.com/Project-Colony/SAM-Colony-Edition/commit/ca6e8d4e08f9f822b7950566129214371b58830a))
* report rejected achievement and stat writes and keep the window responsive ([#26](https://github.com/Project-Colony/SAM-Colony-Edition/issues/26)) ([e91487e](https://github.com/Project-Colony/SAM-Colony-Edition/commit/e91487ea253efcbcfeddf430be9df6e7a0305885))
* **security:** remove the unused shell plugin and enforce a content security policy ([#25](https://github.com/Project-Colony/SAM-Colony-Edition/issues/25)) ([993129b](https://github.com/Project-Colony/SAM-Colony-Edition/commit/993129bf8a24357bd27dcf991fa9e399eef0ee15))
* show Steam's real state after a rejected upload and skip empty uploads ([#29](https://github.com/Project-Colony/SAM-Colony-Edition/issues/29)) ([d486295](https://github.com/Project-Colony/SAM-Colony-Edition/commit/d486295553bf3350f4bb4b18f3c76317aabb181c))


### Internals

* move the Tauri command handlers out of main.rs ([#32](https://github.com/Project-Colony/SAM-Colony-Edition/issues/32)) ([fcfe377](https://github.com/Project-Colony/SAM-Colony-Edition/commit/fcfe37766ebf1b6d99fee4aceb2b30dbb5044047))

## [0.7.0](https://github.com/Project-Colony/SAM-Colony-Edition/compare/v0.6.1...v0.7.0) (2026-10-08)


### Features

* **ci:** sign release assets with the Project-Colony org key ([64d3fca](https://github.com/Project-Colony/SAM-Colony-Edition/commit/64d3fca7e1b9240c811a9655008b6831ef42dcae))
* declare signed releases in the manifest ([2d58487](https://github.com/Project-Colony/SAM-Colony-Edition/commit/2d58487023fe0192618d2e6d6251dfeb1ada47b8))


### Fixes

* **deps:** update rustls to 0.23.45 (RUSTSEC-2026-0285) ([#3](https://github.com/Project-Colony/SAM-Colony-Edition/issues/3)) ([76689b7](https://github.com/Project-Colony/SAM-Colony-Edition/commit/76689b7deafbb28b123b569f4425545ad6982da3))
* drop the misused binary field - per spec it means extract-from-archive, these assets are raw binaries ([b65b771](https://github.com/Project-Colony/SAM-Colony-Edition/commit/b65b7710faf3436c5fd5a91ed7d0a699d873662a))


### Documentation

* state the real licence ([#2](https://github.com/Project-Colony/SAM-Colony-Edition/issues/2)) ([a90192d](https://github.com/Project-Colony/SAM-Colony-Edition/commit/a90192d2b62be905abdc7318d5ce1b228df0e552))

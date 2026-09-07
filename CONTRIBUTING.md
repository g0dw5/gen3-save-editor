# Contributing / 参与贡献

Keep PRs scoped to a capability and its verified formats. Explain observable
behavior and validation, and disclose research gaps. Code and comments use
English. User-facing documentation is maintained in English and Simplified
Chinese; new UI strings require both dictionaries.

Run the checks in README before submitting. Tests must exercise meaningful
binary invariants or user workflows. Use generated fixtures publicly. Keep exact
ROM regression opt-in via local environment variables. Do not commit ROMs,
extracted asset/text collections, commercial guides or personal saves.

For new versions, include table/codec evidence, exact fingerprint, capabilities,
boundary cases and an emulator round trip on a disposable save. Record reference
source URLs and offsets instead of copying upstream implementations without
checking their license. Do not make unsupported adapters pass by disabling hash
checks or silently substituting another version's data.

新增功能请同时说明实际行为、测试结果和未验证范围。新版本适配必须提供精确
指纹及解析证据；公开测试使用合成数据，真实 ROM 与个人存档保留在本地。

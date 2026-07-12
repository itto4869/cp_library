# cp_library

C++17向けの競技プログラミング用ヘッダーライブラリです。`include` を
インクルードパスに追加して使います。

## 出力ユーティリティ

```cpp
#include "cp/io.hpp"

int main() {
    cp::println(10, "apples", 20);       // 10 apples 20
    cp::println_fixed(3, 1.23456, 2.0);  // 1.235 2.000
}
```

- `cp::print(...)`: 任意個の値を半角スペース区切りで出力
- `cp::println(...)`: 同様に出力し、最後に改行
- `cp::print_fixed(digits, ...)`: 小数点以下 `digits` 桁の固定小数点表示
- `cp::println_fixed(digits, ...)`: 同様に出力し、最後に改行
- 各関数の `_to` 版（例: `println_to(stream, ...)`）: 出力先を指定

`print_fixed` / `println_fixed` は呼び出し後にストリームの書式設定を元へ
戻すため、その後の通常出力には影響しません。

## ビルドとテスト

```sh
cmake -S . -B build
cmake --build build
ctest --test-dir build --output-on-failure
```

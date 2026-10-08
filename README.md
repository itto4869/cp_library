# cp_library

競技プログラミング用の Rust ライブラリです。よく使うアルゴリズム、データ構造、数値計算、グリッド補助関数、出力補助をまとめています。

## 環境

- Rust edition: 2024
- Rust version: 1.89.0
- 依存クレート: `ac-library-rs = "0.2.0"`（遅延セグメント木用 MapMonoid）

## 使い方

別プロジェクトからローカル依存として使う場合:

```toml
[dependencies]
cp_library = { path = "../cp_library" }
```

このリポジトリ内でテストする場合:

```sh
cargo test
```

## モジュール一覧

| モジュール | 内容 |
| --- | --- |
| `algorithm` | LIS |
| `data_structure` | 遅延セグメント木用 MapMonoid、IntervalSet、Binary Trie（u64・任意長）、Trie 木、implicit treap、遅延伝搬・反転可能 RBST、重み付き Union-Find |
| `graph` | ダイクストラ法、Functional Graph |
| `grid` | 4 近傍・8 近傍 |
| `math` | 基数変換、組み合わせ、gcd/lcm、素数列挙、素因数分解 |
| `utils` | Yes/No 出力補助 |

## Algorithm

### LIS

`lis` は狭義単調増加部分列の最大長を返します。

```rust
use cp_library::lis;

let a = vec![1, 3, 5, 2, 4, 6];
assert_eq!(lis(&a), 4);
```

- パス: `cp_library::lis` または `cp_library::algorithm::lis`
- 計算量: `O(n log n)`

## Data Structure

### 遅延セグメント木用 MapMonoid

`data_structure::lazy_segtree_map_monoid` は [ac-library-rs の MapMonoid](https://docs.rs/ac-library-rs/0.2.0/ac_library/lazysegtree/trait.MapMonoid.html) を実装する型を提供します。利用側でも `ac-library-rs = "0.2.0"` を依存に追加してください。

| 型 | 更新 / 取得 | 作用 `F` | 集約値 `S` |
| --- | --- | --- | --- |
| `RangeAddSum<T = i64>` | 区間加算 / 和 | `T` | `SumLen<T>` |
| `RangeAssignSum<T = i64>` | 区間代入 / 和 | `Option<T>` | `SumLen<T>` |
| `RangeAffineSum<T = i64>` | `x → a*x+b` / 和 | `(T, T)` | `SumLen<T>` |
| `RangeAddMin` | 区間加算 / 最小値 | `i64` | `Option<i64>` |
| `RangeAddMax` | 区間加算 / 最大値 | `i64` | `Option<i64>` |
| `RangeAssignMin` | 区間代入 / 最小値 | `Option<i64>` | `Option<i64>` |
| `RangeAssignMax` | 区間代入 / 最大値 | `Option<i64>` | `Option<i64>` |

```rust
use ac_library::{LazySegtree, ModInt998244353 as Mint};
use cp_library::data_structure::lazy_segtree_map_monoid::{
    RangeAddSum, RangeAssignMin, RangeAffineSum, SumLen,
};

let leaves = vec![1, 2, 3].into_iter().map(SumLen::new).collect::<Vec<_>>();
let mut sums = LazySegtree::<RangeAddSum>::from(leaves);
sums.apply_range(0..2, 10);
assert_eq!(sums.prod(0..3).sum, 26);

let mut mins = LazySegtree::<RangeAssignMin>::from(vec![Some(3), Some(7)]);
mins.apply_range(.., Some(0));
assert_eq!(mins.all_prod(), Some(0));

let mut affine = LazySegtree::<RangeAffineSum<Mint>>::from(
    vec![SumLen::new(Mint::new(2)); 3],
);
affine.apply_range(.., (Mint::new(3), Mint::new(1)));
assert_eq!(affine.all_prod().sum.val(), 21);
```

- `SumLen::new(value)` は和が `value`、長さが 1 の葉を作ります。集約結果は公開フィールド `sum` / `len` から取得できます。空区間の単位元は和 0・長さ 0 です。
- 最小値・最大値の葉は `Some(value)`、空区間は `None` です。空区間に作用しても `None` を保ちます。
- 代入作用の `None` は何もしない操作です。0 を代入するときは `Some(0)` を使います。
- **`LazySegtree::new(n)` は空区間の単位元で初期化されます。** 値 0 の配列を作る場合も `vec![SumLen::new(0); n]` や `vec![Some(0); n]` から構築してください。
- `composition(f, g)` は `g` の後に `f` を適用します。アフィン作用では `(a_f*a_g, a_f*b_g+b_f)` です。
- 和の型 `T` は `Copy + From<i64> + Add<Output = T> + Mul<Output = T>` を満たす半環を想定します。`i64`、`i128`、ACL の modint を使用できます。整数の和・積・作用の合成は中間値も型の範囲内、要素数は `i64` の範囲内で使用してください。
- 各 MapMonoid 演算は `O(1)`（`T` の演算を定数時間とする）。セグメント木の区間更新・区間取得は `O(log n)` です。

### IntervalSet

`intervalset` モジュールの `IntervalSet` は、`i64` を端点とする半開区間 `[l, r)` の集合を `BTreeSet` で管理します。重なる区間や隣接する区間は自動で結合します。

```rust
use cp_library::data_structure::intervalset::IntervalSet;

let mut set = IntervalSet::new();
set.insert(1, 4);
set.insert(4, 8);
assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 8)]);
set.remove(3, 6);
assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 3), (6, 8)]);
assert!(set.contains(2));
assert!(!set.contains(3));
```

| API | 説明 |
| --- | --- |
| `new()` / `default()` | 空の区間集合を作成 |
| `insert(l, r)` | `[l, r)` を追加し、重複・隣接区間を結合 |
| `remove(l, r)` | `[l, r)` を削除し、必要なら区間を分割 |
| `contains(point)` | 点がいずれかの区間に含まれるか判定 |
| `contains_range(l, r)` | `[l, r)` 全体が覆われているか判定。空区間は `true` |
| `interval_containing(point)` | 点を含む区間を `Option<(i64, i64)>` で返す |
| `iter()` | 左端の昇順で `(l, r)` を列挙 |
| `len()` / `is_empty()` | 保持する区間数 / 空判定 |
| `clear()` | 全区間を削除 |

空区間の追加・削除は何も変更せず、`l > r` は panic します。端点に `i64::MIN` / `i64::MAX` も使えますが、右端を含まないため `i64::MAX` 自体を覆うことはできません。重複追加は登録数を増やさず、削除は指定範囲を集合から取り除きます。

区間数を `n`、操作で影響する区間数を `k` として、追加・削除は `O((k + 1) log(n + 1))`、包含判定は `O(log(n + 1))`、区間数・空判定は `O(1)`、全区間の列挙は `O(n)` です。保持空間は `O(n)` です。

### BinaryTrie

`u64` 全域を扱う 64 ビットの Binary Trie です。重複を含む整数の集合を管理します。

```rust
use cp_library::data_structure::binary_trie::BinaryTrie;

let mut trie = BinaryTrie::new();
trie.insert(5);
trie.insert(5);
trie.insert(10);
assert_eq!(trie.count(5), 2);
assert_eq!(trie.count_less(5), 2); // 5 以下
assert_eq!(trie.count_greater(5), 3); // 5 以上
assert_eq!(trie.min_xor(3), Some(6)); // 5 ^ 3
assert_eq!(trie.max_xor(3), Some(9)); // 10 ^ 3
assert!(trie.remove(5));
assert_eq!(trie.count(5), 1);
```

| API | 説明 |
| --- | --- |
| `new()` / `default()` | 空の Binary Trie を作成 |
| `insert(value)` | 1 件追加 |
| `remove(value)` | 1 件削除し、存在した場合 `true` |
| `count(value)` / `contains(value)` | 登録数 / 存在判定 |
| `count_less(value)` / `count_greater(value)` | 指定値以下 / 以上の個数。等しい値と重複登録を含む |
| `min_xor(x)` / `max_xor(x)` | 登録値との XOR の最小値 / 最大値。空なら `None` |
| `len()` / `is_empty()` | 重複込みの要素数 / 空判定 |
| `clear()` | 全要素と内部ノードを削除 |

`min_xor` / `max_xor` が返すのは XOR の計算結果です。探索・挿入・削除は `O(64)`、要素数・空判定は `O(1)` です。削除済みの経路は再利用のため保持するため、空間は過去に挿入した異なる値の数を `D` として `O(1 + 64D)` です。`clear()` で内部ノードを解放できます。

### ArbitraryBinaryTrie

64 ビットを超える非負整数を扱う任意長の Binary Trie です。上位ビットから並べた `&[bool]` を渡します。ビット幅は挿入時に自動で拡張され、異なる長さの値を混在できます。外部クレートは不要です。

```rust
use cp_library::data_structure::arbitrary_binary_trie::ArbitraryBinaryTrie;

let mut trie = ArbitraryBinaryTrie::new();
let mut large = vec![false; 201];
large[0] = true; // 2^200
trie.insert(&large);
trie.insert(&[true]); // 1
trie.insert(&[false, true]); // 先頭の 0 は無視するので、これも 1

assert_eq!(trie.count(&[true]), 2);
assert_eq!(trie.count_less(&[true]), 2); // 1 以下
assert_eq!(trie.count_greater(&[true]), 3); // 1 以上
assert_eq!(trie.min_xor(&[]), Some(vec![true])); // 1 XOR 0
assert_eq!(trie.max_xor(&[]), Some(large.clone()));
assert!(trie.remove(&large));
```

- `new()` / `default()`、`len()` / `is_empty()`、`clear()` を提供します。
- `insert(bits)` / `remove(bits)` は 1 件ずつ追加・削除します。`remove` は存在した場合に `true` を返します。
- `count(bits)` / `contains(bits)` は先頭の 0 を無視して登録数・存在を判定します。空スライスと 0 だけの列は数値 0 です。
- `count_less(bits)` / `count_greater(bits)` は指定値以下 / 以上の個数を返します。どちらも等しい値と重複登録を含みます。時間 `O(L + W)`、追加空間 `O(1)` です。
- `min_xor(x)` / `max_xor(x)` は XOR の数値としての最小値・最大値を `Option<Vec<bool>>` で返します。空の集合では `None`、結果が 0 なら `Some(vec![])`、それ以外は先頭の 0 を除いた上位ビット順です。登録値より長い検索値も使えます。
- 入力長を `L`、過去に挿入した値の最大有効ビット幅を `W` として、挿入・削除・検索・XOR 検索は `O(L + W)`、要素数・空判定は `O(1)` です。
- 過去に挿入した異なる値の数を `D` として空間は `O(1 + D * W)`。削除済みの経路は保持し、`clear()` で内部ノードとビット幅をリセットします。

### Trie

Unicode の `char` 単位で文字列を管理する Trie 木です。重複登録と空文字列に対応し、Unicode の正規化は行いません。

```rust
use cp_library::data_structure::trie::Trie;

let mut trie = Trie::new();
trie.insert("app");
trie.insert("apple");
trie.insert("app");
assert_eq!(trie.count("app"), 2);
assert_eq!(trie.count_less("app"), 2); // 辞書順で "app" 以下
assert_eq!(trie.count_greater("app"), 3); // 辞書順で "app" 以上
assert_eq!(trie.prefix_count("ap"), 3);
assert!(trie.starts_with("ap"));
assert!(!trie.contains("ap"));
assert!(trie.remove("app"));
assert_eq!(trie.count("app"), 1);
```

| API | 説明 |
| --- | --- |
| `new()` / `default()` | 空の Trie を作成 |
| `insert(word)` | 文字列を 1 件追加 |
| `remove(word)` | 完全一致する文字列を 1 件削除し、存在した場合 `true` |
| `count(word)` / `contains(word)` | 完全一致する登録数 / 存在判定 |
| `count_less(word)` / `count_greater(word)` | 辞書順で指定文字列以下 / 以上の個数。等しい文字列と重複登録を含む |
| `prefix_count(prefix)` / `starts_with(prefix)` | 接頭辞に一致する登録数 / 存在判定 |
| `len()` / `is_empty()` | 重複込みの文字列数 / 空判定 |
| `clear()` | 全文字列と内部ノードを削除 |

文字列の引数は `&str` です。空の接頭辞はすべての登録文字列に一致し、空の Trie では `starts_with("")` も `false` です。

`count_less` / `count_greater` の比較順序は Rust の `str` の辞書順と同じです。空文字列は最小で、接頭辞はそれを延長した文字列より小さくなります。言語固有の照合や Unicode 正規化は行いません。これらの操作は各経路の分岐を集計するため、時間 `O(1 + L * (B + 1))`、追加空間 `O(1)` です。

文字数を `L`、最大分岐数を `B` として、検索・挿入・削除は `O(L log(B + 1))`、要素数・空判定は `O(1)` です。空間は過去に登録した異なる接頭辞の数に比例します。削除済みの経路は再利用のため保持し、`clear()` で内部ノードを解放できます。

### ImplicitTreap

添字で操作する列を treap で管理します。半開区間 `[l, r)` の遅延反転に対応しています。

```rust
use cp_library::data_structure::implicit_treap::ImplicitTreap;

let mut treap: ImplicitTreap<_> = (0..5).collect();

treap.insert(2, 99);
assert_eq!(treap.to_vec(), vec![0, 1, 99, 2, 3, 4]);

treap.reverse(1, 5);
assert_eq!(treap.to_vec(), vec![0, 3, 2, 99, 1, 4]);

assert_eq!(treap.get(2), Some(&2));
assert!(treap.set(2, 20));
assert_eq!(treap.remove(3), Some(99));
assert_eq!(treap.into_vec(), vec![0, 3, 20, 1, 4]);
```

主な API:

| API | 説明 |
| --- | --- |
| `new()` | 空の treap を作成 |
| `with_seed(seed)` | 優先度生成 seed を指定して作成 |
| `len()` / `is_empty()` | 要素数・空判定 |
| `push_front(value)` / `push_back(value)` | 先頭・末尾へ追加 |
| `insert(index, value)` | `index` の位置へ挿入 |
| `remove(index)` | `index` の要素を削除して `Option<T>` を返す |
| `get(index)` / `get_mut(index)` | `index` の参照・可変参照を取得 |
| `set(index, value)` | `index` の値を更新し、成功時 `true` |
| `reverse(l, r)` | 半開区間 `[l, r)` を反転 |
| `to_vec()` | 現在の列を `Vec<T>` として複製 |
| `into_vec()` | treap を消費して `Vec<T>` を返す |
| `clear()` | 空にする |

期待計算量:

- `insert` / `remove` / `get` / `get_mut` / `set` / `reverse`: `O(log n)`
- `to_vec` / `into_vec`: `O(n)`

注意:

- `insert(index, value)` は `index <= len`、`reverse(l, r)` は `l <= r <= len` を満たさない場合 panic します。
- `get` / `get_mut` は遅延反転を伝播するため `&mut self` を取ります。
- `to_vec` は `T: Clone` が必要です。`into_vec` は `T: Clone` 不要です。

### LazyReversibleRbst

列を randomized binary search tree（RBST）で管理します。要素の挿入・削除に加え、半開区間 `[l, r)` への作用、区間積、区間反転を扱えます。順方向と逆方向の区間積を保持するため、文字列連結や写像合成のような非可換モノイドにも対応します。

次は、区間加算と区間和を扱う例です。

```rust
use cp_library::data_structure::lazy_reversible_rbst::{
    LazyReversibleRbst, LazyReversibleRbstSpec,
};

struct RangeAddRangeSum;

impl LazyReversibleRbstSpec for RangeAddRangeSum {
    type Value = i64;
    type Action = i64;

    fn identity() -> Self::Value {
        0
    }

    fn combine(left: &Self::Value, right: &Self::Value) -> Self::Value {
        left + right
    }

    fn apply(action: &Self::Action, value: &Self::Value, len: usize) -> Self::Value {
        value + action * len as i64
    }

    fn compose(new: &Self::Action, old: &Self::Action) -> Self::Action {
        new + old
    }
}

let mut tree: LazyReversibleRbst<RangeAddRangeSum> = (1..=5).collect();

assert_eq!(tree.fold(0, 5), 15);
tree.apply(1, 4, 10);
assert_eq!(tree.all_prod(), 45);

tree.reverse(1, 5);
assert_eq!(tree.to_vec(), vec![1, 5, 14, 13, 12]);

let mut suffix = tree.split_off(3);
assert_eq!(tree.to_vec(), vec![1, 5, 14]);
assert_eq!(suffix.to_vec(), vec![13, 12]);

tree.append(&mut suffix);
assert!(suffix.is_empty());
```

`LazyReversibleRbstSpec` は、値のモノイドと区間作用を定義します。`Value` と `Action` は `Clone` を実装する必要があり、次の規則を満たす必要があります。

- `combine` は結合的で、`identity()` はその左右単位元です。
- `apply(f, combine(x, y), x_len + y_len)` は `combine(apply(f, x, x_len), apply(f, y, y_len))` と等しくなります。
- `compose(new, old)` は `old` を適用した後に `new` を適用する作用を返します。すなわち `apply(compose(new, old), x, len) = apply(new, apply(old, x, len), len)` です。
- `compose` は結合的です。
- `apply` は列の反転と可換でなければなりません。同じ作用を各要素へ一様に適用する区間加算・区間代入・アフィン変換などを想定しています。

これらの規則はコンパイラでは検査されません。

主な API:

| API | 説明 |
| --- | --- |
| `new()` / `with_seed(seed)` | 空の RBST を作成。`with_seed` は乱数 seed を指定 |
| `from_iter_with_seed(iter, seed)` | iterator から指定 seed で構築 |
| `len()` / `is_empty()` | 要素数・空判定 |
| `push_front(value)` / `push_back(value)` | 先頭・末尾へ追加 |
| `insert(index, value)` | `index` の位置へ挿入 |
| `remove(index)` | `index` の要素を削除して `Option<Value>` を返す |
| `get(index)` | `index` の要素への参照を `Option` で返す |
| `set(index, value)` | `index` の値を更新し、成功時 `true` |
| `fold(l, r)` / `prod(l, r)` | `[l, r)` の区間積を返す。`prod` は `fold` の別名 |
| `all_prod()` | 列全体の積を返す |
| `apply(l, r, action)` | `[l, r)` の全要素へ作用を適用 |
| `reverse(l, r)` / `reverse_all()` | 指定区間・列全体を反転 |
| `split_off(index)` | `[index, len)` を分離して新しい RBST として返す |
| `append(other)` | `other` の全要素を末尾へ移動し、`other` を空にする |
| `to_vec()` / `into_vec()` | 現在の列を `Vec<Value>` として返す |
| `clear()` | 空にする |

期待計算量（`LazyReversibleRbstSpec` の各演算と `Value` / `Action` の clone を `O(1)` とした場合）:

- `new` / `len` / `is_empty` / `all_prod` / `reverse_all`: `O(1)`
- 挿入・削除・参照・更新・区間積・区間作用・区間反転・`split_off`: 期待 `O(log n)`
- `append`: 2 つの列の合計長を `n` として期待 `O(log n)`
- iterator または `Vec` からの構築、`to_vec` / `into_vec` / `clone` / `clear`: `O(n)`

注意:

- すべての区間は半開区間 `[l, r)` です。`l <= r <= len` を満たさない場合、`fold` / `prod` / `apply` / `reverse` は panic します。
- 空区間では `fold` / `prod` は `identity()` を返し、`apply` / `reverse` は何もしません。
- `insert(index, value)` と `split_off(index)` は `index <= len` を満たさない場合 panic します。
- 範囲外の `get` / `remove` は `None`、`set` は `false` を返します。
- `get` / `fold` / `prod` / `to_vec` は遅延操作を伝播することがあるため `&mut self` を取ります。
- `new()` は固定 seed を使うため、同じ操作列に対する木の形は再現可能です。期待計算量は擬似乱数列と操作列が独立であることを仮定します。
- 計算量は乱択による期待値であり、木が偏った場合の最悪計算量は `O(n)` です。

### WeightedDsu

重み付き Union-Find です。`merge(x, y, w)` は `weight(y) = weight(x) + w` となるように集合を併合します。

```rust
use cp_library::data_structure::weighted_dsu::WeightedDsu;

let mut dsu = WeightedDsu::<i64>::new(4);

assert!(dsu.merge(0, 1, 2)); // weight(1) = weight(0) + 2
assert!(dsu.merge(1, 2, 3)); // weight(2) = weight(1) + 3

assert_eq!(dsu.diff(0, 2), Some(5));
assert_eq!(dsu.diff(2, 0), Some(-5));
assert!(dsu.same(0, 2));
assert_eq!(dsu.size(0), 3);
assert_eq!(dsu.diff(0, 3), None);
```

主な API:

| API | 説明 |
| --- | --- |
| `new(size)` | `size` 個の要素で初期化 |
| `find(x)` | 根を返す |
| `weight(x)` | 根から `x` までの重みを返す |
| `diff(x, y)` | 同じ集合なら `weight(y) - weight(x)` を返す |
| `same(x, y)` | 同じ集合か判定 |
| `merge(x, y, w)` | `weight(y) = weight(x) + w` として併合 |
| `size(x)` | `x` を含む集合のサイズ |

計算量: ならし `O(alpha(n))`

型 `T` は `Copy + Default + Add + Sub + Neg` を満たす必要があります。

## Graph

### FunctionalGraph

各頂点からちょうど 1 本の辺が出る有向グラフです。`next[v]` を頂点 `v` の行き先として構築します。

```rust
use cp_library::graph::FunctionalGraph;

let graph = FunctionalGraph::new(vec![1, 2, 1, 4, 3, 4, 6]);
assert_eq!(graph.cycles(), vec![vec![1, 2], vec![3, 4], vec![6]]);
assert_eq!(graph.next(0), 1);
```

- `new(next: Vec<usize>)`: 構築。頂点番号は `0..next.len()` で、範囲外の行き先は panic します。空グラフも扱えます。
- `cycles() -> Vec<Vec<usize>>`: すべてのサイクルを返します。流入するだけの頂点は含みません。
- 各サイクルは最小の頂点番号から辺の向きに並び、末尾に始点を重複させません。サイクル同士は最小頂点番号の昇順です。自己ループは要素 1 個の列になります。
- `next(vertex)`: 行き先を返します。範囲外の頂点は panic します。
- `len()` / `is_empty()`: 頂点数 / 空判定。
- 計算量: 構築・サイクル抽出は時間 `O(n)`、サイクル抽出の追加空間は `O(n)`。各アクセサは `O(1)`。

### dijkstra

`Vec<Vec<(usize, usize)>>` の隣接リストと始点を受け取り、各頂点への最短距離を `Vec<usize>` で返します。各要素は `(行き先の頂点, コスト)` です。

```rust
use cp_library::graph::dijkstra;

let graph = vec![vec![(1, 4), (2, 1)], vec![], vec![(1, 2)], vec![]];
assert_eq!(dijkstra(&graph, 0), vec![0, 3, 1, usize::MAX]);
```

- 引数: `graph: &[Vec<(usize, usize)>]`, `start: usize`（Vec は `&graph` で渡せます）
- 頂点番号は 0 始まり、コストは非負整数です。無向グラフでは両方向の辺を登録してください。
- 到達不能、または距離が `usize::MAX` 以上の場合は `usize::MAX` を返します。加算は飽和演算です。
- 始点や辺の行き先が範囲外の場合は panic します。空グラフも始点が存在しないため panic します。
- 計算量: 時間 `O(V + E log(E + 1))`、追加空間 `O(V + E)`

## Grid

### neighbors4 / neighbors8

グリッド境界内の近傍座標を返します。

```rust
use cp_library::grid::{neighbors4, neighbors8};

assert_eq!(neighbors4(0, 0, 3, 3), vec![(0, 1), (1, 0)]);

let n8 = neighbors8(1, 1, 3, 3);
assert_eq!(n8.len(), 8);
assert!(n8.contains(&(0, 0)));
assert!(n8.contains(&(2, 2)));
```

- 引数: `(r, c, h, w)`
- 戻り値: `Vec<(usize, usize)>`

## Math

### convert_base

2 から 36 進数までの基数変換を行います。入力は `Display` 実装型を受け取れます。

```rust
use cp_library::math::base_conversion::convert_base;

assert_eq!(convert_base("255", 10, 16).unwrap(), "ff");
assert_eq!(convert_base("ff", 16, 10).unwrap(), "255");
assert_eq!(convert_base(-10, 10, 2).unwrap(), "-1010");
```

- パス: `cp_library::math::base_conversion::convert_base`
- 変換元・変換先の基数は `2..=36`
- 不正な基数や文字がある場合は `Err(&'static str)` を返します。

### Combination

階乗と逆階乗を前計算して、素数 mod 上の `nCr`、`nPr`、`nHr` を計算します。

```rust
use cp_library::math::combinations::Combination;

const MOD: u64 = 1_000_000_007;
let comb = Combination::new(100, MOD);

assert_eq!(comb.n_c_r(5, 2), 10);
assert_eq!(comb.n_p_r(5, 2), 20);
assert_eq!(comb.n_h_r(3, 2), 6);
assert_eq!(comb.fact(5), 120);
```

- パス: `cp_library::math::combinations::Combination`
- `modulo` は Fermat の小定理で逆元を計算するため、素数を指定してください。
- `new(max_n, modulo)` で `0..=max_n` まで前計算します。

計算量:

- 前計算: `O(max_n log modulo)`
- 各クエリ: `O(1)`

### gcd / lcm

整数型の最大公約数・最小公倍数を計算します。

```rust
use cp_library::math::numeric::{gcd, lcm, GCD};

assert_eq!(gcd(12u64, 18), 6);
assert_eq!(lcm(12u64, 18), 36);
assert_eq!(12usize.gcd(18), 6);
assert_eq!(12usize.lcm(18), 36);
```

- パス: `cp_library::math::numeric::{gcd, lcm, GCD}`
- 対応型: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`

### prime_factorize

決定的 Miller-Rabin 素数判定と Pollard's Rho 法を使って、`u64` を高速に素因数分解します。

```rust
use cp_library::math::prime_factorization::prime_factorize;

assert_eq!(prime_factorize(360), vec![(2, 3), (3, 2), (5, 1)]);
assert_eq!(prime_factorize(1), vec![]);
```

- 戻り値: 素数の昇順に並んだ `(素因数, 指数)` の `Vec`
- 対応範囲: `1..=u64::MAX`（`0` は panic）
- 素数判定は `u64` 全域で決定的です。

### enumerate_primes

エラトステネスの篩を使い、指定値以下の素数を列挙します。

```rust
use cp_library::math::prime_enumeration::enumerate_primes;

assert_eq!(enumerate_primes(10), vec![2, 3, 5, 7]);
assert_eq!(enumerate_primes(1), vec![]);
```

- 戻り値: 指定値以下の素数を昇順に並べた `Vec<usize>`
- 計算量: 時間 `O(n log log n)`、空間 `O(n)`

## Utils

### yes_no / yes_no_custom

bool を `"Yes"` / `"No"` などの文字列へ変換します。マクロ版はそのまま `println!` します。

```rust
use cp_library::utils::{yes_no, yes_no_custom};

assert_eq!(yes_no(true), "Yes");
assert_eq!(yes_no(false), "No");
assert_eq!(yes_no_custom(true, "YES", "NO"), "YES");

cp_library::yes_no!(true);
cp_library::yes_no_custom!(false, "Possible", "Impossible");
```

関数:

- `yes_no(b) -> &'static str`
- `yes_no_custom(b, yes, no) -> &str`

マクロ:

- `yes_no!(b)`: `Yes` または `No` を出力
- `yes_no_custom!(b, yes, no)`: 指定文字列を出力

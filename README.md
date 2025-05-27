## treegrep

treegrep is a pattern matcher that displays results in a tree structure with an interface to jump to matched text.

[![test](https://github.com/4imothy/treegrep/actions/workflows/test.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

[examples](#examples) and [help](#--help).

### supported backends
- *[ripgrep](https://github.com/BurntSushi/ripgrep)*
- *[treegrep](https://github.com/4imothy/treegrep)*

### links
[crates.io](https://crates.io/crates/treegrep) | [GitHub](https://github.com/4imothy/treegrep) | [AUR](https://aur.archlinux.org/packages/treegrep-bin) | [NetBSD](https://pkgsrc.se/sysutils/treegrep)

### installation
- **Cargo:** ```cargo install treegrep```
- **Releases:** Download from [releases](https://github.com/4imothy/treegrep/releases/)
- **Manual:**
  ```
  git clone https://github.com/4imothy/treegrep
  cd treegrep
  cargo build --release
  ```

### examples
<details>
<summary><code>tgrep --regexp \bstruct\s+\w+ --regexp \bimpl\s+\w+ --path src --line-number --count</code></summary>

```
src: 12
├──args.rs: 2
│  ├──16: pub struct ArgInfo {
│  └──22: impl ArgInfo {
├──matcher.rs: 1
│  └──115: impl File {
├──config.rs: 3
│  ├──11: pub struct Characters {
│  ├──25: pub struct Config {
│  └──79: impl Config {
├──formats.rs: 2
│  ├──19: pub struct Chars {
│  └──99: pub struct DisplayRepeater<T>(T, usize);
├──match_system.rs: 10
│  ├──39: pub struct Directory {
│  ├──48: impl Directory {
│  ├──61: pub struct File {
│  ├──68: impl File {
│  ├──107: pub struct Match {
│  ├──113: impl Match {
│  ├──138: pub struct Line {
│  ├──144: impl Line {
│  ├──160:     impl PartialEq for Match {
│  └──167:     impl Debug for Match {
├──output_processor.rs: 2
│  ├──29: impl File {
│  └──100: impl AsUsize for Value {
├──writer.rs: 10
│  ├──21: impl Clone for PrefixComponent {
│  ├──32: pub struct OpenInfo<'a> {
│  ├──41: struct PathDisplay<'a> {
│  ├──125: struct LineDisplay<'a> {
│  ├──217: struct LongBranchDisplay<'a> {
│  ├──256: struct OverviewDisplay {
│  ├──264: impl Entry for OverviewDisplay {
│  ├──270: impl Display for OverviewDisplay {
│  ├──303: impl Directory {
│  └──402: impl File {
├──term.rs: 1
│  └──13: pub struct Term<'a> {
├──options.rs: 2
│  ├──42: pub struct Rg;
│  └──44: impl Options for Rg {
├──menu.rs: 5
│  ├──19: struct PathInfo {
│  ├──26: impl PathInfo {
│  ├──83: pub struct Menu<'a> {
│  ├──100: struct Window {
│  └──106: impl Window {
├──searchers.rs: 5
│  ├──12: struct ShortName(String);
│  ├──14: impl ShortName {
│  ├──21: impl std::fmt::Display for ShortName {
│  ├──27: impl Deref for ShortName {
│  └──89: impl Searchers {
└──errors.rs: 4
   ├──8: pub struct Message {
   ├──22: impl Error for Message {}
   ├──24: impl fmt::Debug for Message {
   └──30: impl fmt::Display for Message {
```
</details>

<details>
<summary><code>tgrep Print src/menu.rs --trim --line-number --char-style=ascii</code></summary>

```
menu.rs
+--9: style::{Print, SetBackgroundColor},
+--325: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--355: Print(self.lines.get(id).unwrap())
+--384: Print(
+--501: Print(config().c.selected_indicator),
+--503: Print(self.lines.get(self.selected_id).unwrap())
+--511: Print(formats::SELECTED_INDICATOR_CLEAR),
+--513: Print(self.lines.get(self.selected_id).unwrap())
+--527: Print(format!(
+--539: Print(format!(
+--552: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden</code></summary>

```
treegrep
├──tests
│  ├──targets
│  │  ├──links_12
│  │  ├──wide_21
│  │  ├──links_22
│  │  ├──deep
│  │  ├──glob_inclusion
│  │  ├──links_21
│  │  ├──file
│  │  ├──links_11
│  │  ├──wide_12
│  │  ├──colon
│  │  ├──line_number
│  │  ├──glob_exclusion
│  │  └──max_depth
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──file_system.rs
│  ├──tests.rs
│  └──utils.rs
├──benchmarks
│  ├──runner
│  └──times
├──.github
│  └──workflows
│     ├──update_readme_and_completions.yml
│     ├──update_readme
│     ├──test.yml
│     └──cr.yml
├──.git
│  ├──refs
│  │  ├──remotes
│  │  │  └──origin
│  │  │     ├──main
│  │  │     └──HEAD
│  │  └──heads
│  │     └──main
│  ├──info
│  │  └──exclude
│  ├──logs
│  │  ├──refs
│  │  │  ├──remotes
│  │  │  │  └──origin
│  │  │  │     ├──main
│  │  │  │     └──HEAD
│  │  │  └──heads
│  │  │     └──main
│  │  └──HEAD
│  ├──objects
│  │  ├──f3
│  │  │  └──2faf861c81658b97a6c38b6ddf1c1690691a27
│  │  ├──51
│  │  │  └──a4597c1d01f9cd45e689880c496dd2d87a302c
│  │  ├──ec
│  │  │  ├──bdfa3a630e183cf4e4ac25aca1adb1d010c4b6
│  │  │  └──521305a5fa24552e97cee894b82d82403798cb
│  │  ├──47
│  │  │  └──92efb350db8807397ff8bd69d4daad51fc3834
│  │  ├──44
│  │  │  └──8851afb707a77bcd11756bc2cbeabf96953954
│  │  ├──28
│  │  │  └──ea9e27550de308da32418060642020ef2783ff
│  │  ├──24
│  │  │  └──66719c50be54a9fe5248a73e96d77b3d7cdfd6
│  │  ├──eb
│  │  │  └──529f6c6a4d651746d8b95d0bd4ff3812a06c36
│  │  ├──93
│  │  │  ├──b0ee6c0e59b86502ca5af180eecff397deb143
│  │  │  └──1ed5666f1d701e4499d8f893bf822784c0de65
│  │  ├──77
│  │  │  └──b75680b87e75a684bd7b7cc51fc1b94e73de85
│  │  ├──3a
│  │  │  └──6c63d0bd2a26135f0f81b32f7a14cfa044c62e
│  │  ├──10
│  │  │  └──a16fabe868b68c1827ef98459392c3953a40c8
│  │  ├──e9
│  │  │  └──00b706b1d74a31ce539f5aee876031ced8fcc2
│  │  ├──08
│  │  │  └──2ed7c79519d800438bf99e2ae7d390626a1d50
│  │  ├──fa
│  │  │  └──96975c4985602ca7be2a3d9f323733b2a519f7
│  │  ├──1f
│  │  │  └──b50407b719793d6f56e3a01221ecf3107c7153
│  │  ├──36
│  │  │  └──116a414838491a652bbbaf291479c3153cbbc4
│  │  ├──f7
│  │  │  └──1b56cc073c21d56769636e04e501e79d96e7e8
│  │  ├──96
│  │  │  └──24c137abe03239a16a64bd36a8f6f2bd386ee0
│  │  ├──f6
│  │  │  └──7ed9013e5818ac3d54a2f0be81cfc43c87a1d9
│  │  ├──50
│  │  │  └──6976e28fe073047e47b4fb3bcaecda28a9857e
│  │  ├──8e
│  │  │  └──0e4d74a276792eed2166ef0787544382e2afc8
│  │  ├──c7
│  │  │  └──53abb543c8fb7ad3163cef5270c14e97aae4e4
│  │  ├──5a
│  │  │  └──bf277f863ed5bd47514928d3fc27152622ff4e
│  │  ├──13
│  │  │  ├──6bfe929c11b119cd6f188bd3643a3d16718c6e
│  │  │  └──9555d3fbe5afd36522aad822b5d08e8c86208a
│  │  ├──62
│  │  │  ├──e9ccd8b414c1bbed4fc916fee0de1e366c1a44
│  │  │  ├──8189ac5ee86b961b3a809ed5e14eb46555ecf5
│  │  │  └──25acebc25c0d9c52a32db33dd44d469c11fc5c
│  │  ├──4e
│  │  │  └──659388bcc690da422761961e82791d62a4b2df
│  │  ├──cf
│  │  │  ├──77e04c46d104adb6acdd54bdeeb3ed8ee5e71c
│  │  │  └──07b87aba28552e7c613326e36362ae21449ac7
│  │  ├──c6
│  │  │  └──78e1b9c9cfe6502725c128dc3463ec3df821a4
│  │  ├──ce
│  │  │  └──d0364cc3101f1e00563c4bc21bf3e780874b7f
│  │  ├──a0
│  │  │  └──7ea1db76860a69cb6d5780f5a6405de2cdea4f
│  │  ├──8f
│  │  │  ├──5fceac2a46a1659751784c8a1039df9733e52e
│  │  │  └──82022e2be484754421763534d0c6a7f0f3ac13
│  │  ├──9f
│  │  │  └──f82ab0bf35453d12679dc81c6fc2825fdfd3fd
│  │  ├──61
│  │  │  └──cb5bbfd97bf696a42d9cc0fd5c8e24805a5db6
│  │  ├──8b
│  │  │  └──b8ad0336f72543c1ea1164c43df3b72a011603
│  │  ├──fd
│  │  │  └──4b29a626802f62e6804951193b36d97f8dd725
│  │  ├──99
│  │  │  └──5b21bfa880f8bbaaeadf467caf055b89df49b9
│  │  ├──d1
│  │  │  └──6811e515346c3bb76739244272cb757fe9a212
│  │  ├──21
│  │  │  └──6f9f63f9bd0bc3777b1697dff97ce4c44b983e
│  │  ├──3c
│  │  │  └──5f2c7bc9000bfd0b9efbd9c70ca04189b770a2
│  │  ├──ef
│  │  │  └──2abbf9c7599a2df460d077bc54fc31fc4546e5
│  │  ├──b2
│  │  │  └──35d73dbad19435d08b8e90b802e588b7ed8357
│  │  ├──1a
│  │  │  └──6f62de714c127016b8348e7ae82f7e2f162eb7
│  │  ├──ee
│  │  │  └──f8e10ea7390cf57af6e60b6f3d003770746a1b
│  │  ├──66
│  │  │  └──9dd5d1064df1780b53d4350a5b2ad61c460b28
│  │  ├──5d
│  │  │  └──ad73b89996d637a91170b90afca5be3552917b
│  │  ├──aa
│  │  │  ├──cfd395ab794e0593f7dc707f2d92d17f461a61
│  │  │  └──271b74d0100e00a8d6b3b5ab71504a85af0f17
│  │  ├──b9
│  │  │  └──39cf9adb983f1d891f66afadde383c80500633
│  │  ├──2c
│  │  │  ├──29e9395c3bc59abf6f1ac602e0ad3e5af0d86f
│  │  │  └──c57ba9228b035760114e5cde2f7f1e1058d354
│  │  └──a6
│  │     └──d86f58a04c9c02cb549b2f0fd18bd3409c45fe
│  ├──hooks
│  │  ├──push-to-checkout.sample
│  │  ├──post-update.sample
│  │  ├──applypatch-msg.sample
│  │  ├──commit-msg.sample
│  │  ├──prepare-commit-msg.sample
│  │  ├──pre-commit.sample
│  │  ├──pre-rebase.sample
│  │  ├──pre-receive.sample
│  │  ├──pre-merge-commit.sample
│  │  ├──update.sample
│  │  ├──pre-push.sample
│  │  ├──pre-applypatch.sample
│  │  ├──fsmonitor-watchman.sample
│  │  └──sendemail-validate.sample
│  ├──description
│  ├──config.worktree
│  ├──config
│  ├──shallow
│  ├──HEAD
│  ├──index
│  └──FETCH_HEAD
├──src
│  ├──args.rs
│  ├──matcher.rs
│  ├──config.rs
│  ├──formats.rs
│  ├──match_system.rs
│  ├──log.rs
│  ├──main.rs
│  ├──output_processor.rs
│  ├──writer.rs
│  ├──term.rs
│  ├──options.rs
│  ├──menu.rs
│  ├──searchers.rs
│  └──errors.rs
├──completions
│  ├──_tgrep.ps1
│  ├──tgrep.bash
│  ├──_tgrep
│  ├──tgrep.elv
│  └──tgrep.fish
├──todos.md
├──build.rs
├──README.md
├──LICENSE
├──Cargo.toml
├──Cargo.lock
└──.gitignore
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden</code></summary>

```
treegrep
├──tests
│  ├──targets
│  │  ├──links_12, wide_21, links_22, deep, glob_inclusion
│  │  ├──links_21, file, links_11, wide_12, colon
│  │  └──line_number, glob_exclusion, max_depth
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  └──file_system.rs, tests.rs, utils.rs
├──benchmarks
│  └──runner, times
├──.github
│  └──workflows
│     └──update_readme_and_completions.yml, update_readme, test.yml, cr.yml
├──.git
│  ├──refs
│  │  ├──remotes
│  │  │  └──origin
│  │  │     └──main, HEAD
│  │  └──heads
│  │     └──main
│  ├──info
│  │  └──exclude
│  ├──logs
│  │  ├──refs
│  │  │  ├──remotes
│  │  │  │  └──origin
│  │  │  │     └──main, HEAD
│  │  │  └──heads
│  │  │     └──main
│  │  └──HEAD
│  ├──objects
│  │  ├──f3
│  │  │  └──2faf861c81658b97a6c38b6ddf1c1690691a27
│  │  ├──51
│  │  │  └──a4597c1d01f9cd45e689880c496dd2d87a302c
│  │  ├──ec
│  │  │  └──bdfa3a630e183cf4e4ac25aca1adb1d010c4b6, 521305a5fa24552e97cee894b82d82403798cb
│  │  ├──47
│  │  │  └──92efb350db8807397ff8bd69d4daad51fc3834
│  │  ├──44
│  │  │  └──8851afb707a77bcd11756bc2cbeabf96953954
│  │  ├──28
│  │  │  └──ea9e27550de308da32418060642020ef2783ff
│  │  ├──24
│  │  │  └──66719c50be54a9fe5248a73e96d77b3d7cdfd6
│  │  ├──eb
│  │  │  └──529f6c6a4d651746d8b95d0bd4ff3812a06c36
│  │  ├──93
│  │  │  └──b0ee6c0e59b86502ca5af180eecff397deb143, 1ed5666f1d701e4499d8f893bf822784c0de65
│  │  ├──77
│  │  │  └──b75680b87e75a684bd7b7cc51fc1b94e73de85
│  │  ├──3a
│  │  │  └──6c63d0bd2a26135f0f81b32f7a14cfa044c62e
│  │  ├──10
│  │  │  └──a16fabe868b68c1827ef98459392c3953a40c8
│  │  ├──e9
│  │  │  └──00b706b1d74a31ce539f5aee876031ced8fcc2
│  │  ├──08
│  │  │  └──2ed7c79519d800438bf99e2ae7d390626a1d50
│  │  ├──fa
│  │  │  └──96975c4985602ca7be2a3d9f323733b2a519f7
│  │  ├──1f
│  │  │  └──b50407b719793d6f56e3a01221ecf3107c7153
│  │  ├──36
│  │  │  └──116a414838491a652bbbaf291479c3153cbbc4
│  │  ├──f7
│  │  │  └──1b56cc073c21d56769636e04e501e79d96e7e8
│  │  ├──96
│  │  │  └──24c137abe03239a16a64bd36a8f6f2bd386ee0
│  │  ├──f6
│  │  │  └──7ed9013e5818ac3d54a2f0be81cfc43c87a1d9
│  │  ├──50
│  │  │  └──6976e28fe073047e47b4fb3bcaecda28a9857e
│  │  ├──8e
│  │  │  └──0e4d74a276792eed2166ef0787544382e2afc8
│  │  ├──c7
│  │  │  └──53abb543c8fb7ad3163cef5270c14e97aae4e4
│  │  ├──5a
│  │  │  └──bf277f863ed5bd47514928d3fc27152622ff4e
│  │  ├──13
│  │  │  └──6bfe929c11b119cd6f188bd3643a3d16718c6e, 9555d3fbe5afd36522aad822b5d08e8c86208a
│  │  ├──62
│  │  │  └──e9ccd8b414c1bbed4fc916fee0de1e366c1a44, 8189ac5ee86b961b3a809ed5e14eb46555ecf5, 25acebc25c0d9c52a32db33dd44d469c11fc5c
│  │  ├──4e
│  │  │  └──659388bcc690da422761961e82791d62a4b2df
│  │  ├──cf
│  │  │  └──77e04c46d104adb6acdd54bdeeb3ed8ee5e71c, 07b87aba28552e7c613326e36362ae21449ac7
│  │  ├──c6
│  │  │  └──78e1b9c9cfe6502725c128dc3463ec3df821a4
│  │  ├──ce
│  │  │  └──d0364cc3101f1e00563c4bc21bf3e780874b7f
│  │  ├──a0
│  │  │  └──7ea1db76860a69cb6d5780f5a6405de2cdea4f
│  │  ├──8f
│  │  │  └──5fceac2a46a1659751784c8a1039df9733e52e, 82022e2be484754421763534d0c6a7f0f3ac13
│  │  ├──9f
│  │  │  └──f82ab0bf35453d12679dc81c6fc2825fdfd3fd
│  │  ├──61
│  │  │  └──cb5bbfd97bf696a42d9cc0fd5c8e24805a5db6
│  │  ├──8b
│  │  │  └──b8ad0336f72543c1ea1164c43df3b72a011603
│  │  ├──fd
│  │  │  └──4b29a626802f62e6804951193b36d97f8dd725
│  │  ├──99
│  │  │  └──5b21bfa880f8bbaaeadf467caf055b89df49b9
│  │  ├──d1
│  │  │  └──6811e515346c3bb76739244272cb757fe9a212
│  │  ├──21
│  │  │  └──6f9f63f9bd0bc3777b1697dff97ce4c44b983e
│  │  ├──3c
│  │  │  └──5f2c7bc9000bfd0b9efbd9c70ca04189b770a2
│  │  ├──ef
│  │  │  └──2abbf9c7599a2df460d077bc54fc31fc4546e5
│  │  ├──b2
│  │  │  └──35d73dbad19435d08b8e90b802e588b7ed8357
│  │  ├──1a
│  │  │  └──6f62de714c127016b8348e7ae82f7e2f162eb7
│  │  ├──ee
│  │  │  └──f8e10ea7390cf57af6e60b6f3d003770746a1b
│  │  ├──66
│  │  │  └──9dd5d1064df1780b53d4350a5b2ad61c460b28
│  │  ├──5d
│  │  │  └──ad73b89996d637a91170b90afca5be3552917b
│  │  ├──aa
│  │  │  └──cfd395ab794e0593f7dc707f2d92d17f461a61, 271b74d0100e00a8d6b3b5ab71504a85af0f17
│  │  ├──b9
│  │  │  └──39cf9adb983f1d891f66afadde383c80500633
│  │  ├──2c
│  │  │  └──29e9395c3bc59abf6f1ac602e0ad3e5af0d86f, c57ba9228b035760114e5cde2f7f1e1058d354
│  │  └──a6
│  │     └──d86f58a04c9c02cb549b2f0fd18bd3409c45fe
│  ├──hooks
│  │  ├──push-to-checkout.sample, post-update.sample, applypatch-msg.sample, commit-msg.sample, prepare-commit-msg.sample
│  │  ├──pre-commit.sample, pre-rebase.sample, pre-receive.sample, pre-merge-commit.sample, update.sample
│  │  └──pre-push.sample, pre-applypatch.sample, fsmonitor-watchman.sample, sendemail-validate.sample
│  ├──description, config.worktree, config, shallow, HEAD
│  └──index, FETCH_HEAD
├──src
│  ├──args.rs, matcher.rs, config.rs, formats.rs, match_system.rs
│  ├──log.rs, main.rs, output_processor.rs, writer.rs, term.rs
│  └──options.rs, menu.rs, searchers.rs, errors.rs
├──completions
│  └──_tgrep.ps1, tgrep.bash, _tgrep, tgrep.elv, tgrep.fish
├──todos.md, build.rs, README.md, LICENSE, Cargo.toml
└──Cargo.lock, .gitignore
```
</details>

### *--help*
```
treegrep 0.1.4

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] [positional regexp] [positional target]
       tgrep [OPTIONS] [positional regexp] [positional target] <COMMAND>
Commands:
  completions  generate completions for given shell

Arguments:
  [positional regexp]  the regex expression
  [positional target]  the path to search, if not provided, search the current directory

Options:
  -e, --regexp <>            the regex expression
  -p, --path <>              the path to search, if not provided, search the current directory
      --glob <>              rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
  -s, --searcher <>          executable to do the searching [possible values: rg, tgrep]
      --threads <>           set the appropriate number of threads to use
  -., --hidden               search hidden files
  -n, --line-number          show line number of match
  -f, --files                if a pattern is given hide matched content, otherwise show the files that would be searched
      --links                search linked paths
      --trim                 trim whitespace at the beginning of lines
      --pcre2                enable PCRE2
      --no-ignore            don't use ignore files
  -c, --count                display number of files matched in directory and number of lines matched in a file
      --no-color             don't use colors
      --no-bold              don't bold anything
      --overview             conclude results with an overview
      --max-depth <>         the max depth to search
      --prefix-len <>        number of characters to show before a match [default: 3]
      --max-length <>        set the max length for a matched line
      --long-branch-each <>  number of files to print on each branch [default: 5]
      --char-style <>        style of characters to use [possible values: ascii, single, double, heavy, rounded, none]
      --long-branch          multiple files from the same directory are shown on the same branch
  -m, --menu                 open results in a menu to be edited with $EDITOR
                             navigate through the menu using the following commands:
                              - move up/down: k/j, p/n, up arrow/down arrow
                              - move up/down with a bigger jump: K/J, P/N
                              - move up/down paths: {/}, [/]
                              - move to the start/end: g/G, </>, home/end
                              - move up/down a page: b/f, pageup/pagedown
                              - center cursor: z/l
                              - quit: q, ctrl + c
  -h, --help                 Print help
  -V, --version              Print version

any of the above can be set using the TREEGREP_DEFAULT_OPTS environment variable
```

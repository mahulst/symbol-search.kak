# symbol-search.kak

Search symbols across various languages among files in your current working directory.

Example output:
```bash

$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
         Running `target/debug/kak-symbol-search`
         [Function] SubmissionReview               (/Users/mahulst/projects/symbol-search.kak/test/jsx/test.jsx:76:7)
         [Class]    MyClass                        (/Users/mahulst/projects/symbol-search.kak/test/jsx/test.jsx:73:7)
         [Class]    App                            (/Users/mahulst/projects/symbol-search.kak/test/vue/App.vue:1:1)
         [Interface] Props                          (/Users/mahulst/projects/symbol-search.kak/test/vue/App.vue:12:11)
         [Function] handleClick                    (/Users/mahulst/projects/symbol-search.kak/test/vue/App.vue:20:10)
         [Function] submitForm                     (/Users/mahulst/projects/symbol-search.kak/test/vue/App.vue:24:7)
         [Type]     ButtonVariant                  (/Users/mahulst/projects/symbol-search.kak/test/vue/App.vue:16:6)
         [Struct]   Entity                         (/Users/mahulst/projects/symbol-search.kak/test/main.odin:32:1)
         [Type]     CONST_VAL                      (/Users/mahulst/projects/symbol-search.kak/test/main.odin:6:1)
         [Type]     My_Int                         (/Users/mahulst/projects/symbol-search.kak/test/main.odin:13:1)
         [Type]     int                            (/Users/mahulst/projects/symbol-search.kak/test/main.odin:13:11)
         [Type]     My_Int_Distinct                (/Users/mahulst/projects/symbol-search.kak/test/main.odin:15:1)
         [Enum]     Things                         (/Users/mahulst/projects/symbol-search.kak/test/main.odin:8:1)
         [Method]   main                           (/Users/mahulst/projects/symbol-search.kak/test/main.odin:17:1)
         [Method]   test_method                    (/Users/mahulst/projects/symbol-search.kak/test/main.odin:28:1)
```

## Installation

1. cargo build --release
1. move the binary somewhere that's on your PATH
1. it's intended to be used with kak-fzf

This is how I use it:
```
hook global ModuleLoaded fzf %{
    map -docstring 'symbols' global fzf S ': require-module fzf-symbol; fzf-symbol<ret>'
}

provide-module fzf-symbol %§

declare-option -docstring "" \
str fzf_symbol_search_command 'kak-symbol-search'

define-command -hidden fzf-symbol %{ evaluate-commands %sh{
    cmd="$kak_opt_fzf_symbol_search_command"
    cmd="$cmd 2>/dev/null"
    title="fzf symbol"
    message="find symbols in project.
<ret>: open search result in new buffer."

    preview_cmd=""

    printf "%s\n" "info -title '${title}' '${message}${tmux_keybindings}'"
    [ -n "${kak_client_env_TMUX}" ] && additional_flags="--expect ${kak_opt_fzf_vertical_map:-ctrl-v} --expect ${kak_opt_fzf_horizontal_map:-ctrl-s}"
    printf "%s\n" "fzf -kak-cmd %{evaluate-commands} ${preview_cmd} -fzf-args %{--expect ${kak_opt_fzf_window_map:-ctrl-w} $additional_flags  -n 2} -items-cmd %{$cmd} -filter %{sed -E 's/.*\(([^:]+):([0-9]+):([0-9]+)\).*/edit -existing \1; execute-keys \2g /'}"
}}
§
```

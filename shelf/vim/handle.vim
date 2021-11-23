source $HOME/local/jago/shelf/vim/profile/alec.vim

function! InstrumentHandle(...) abort
    if has('terminal')
        let cmd = 'vertical terminal'
    else
        throw "upgrade vim to version 8.1 or higher"
    endif

    let input = substitute(a:1, '%', expand('%'), '')

    execute cmd input
endfunction

" nmap <silent> <Leader>t :call InstrumentHandle("cargo test --package scratch --features scratch/search")<CR>
" nmap <silent> <Leader>T :call InstrumentHandle("cargo test --package scratch --features scratch/search -- --nocapture")<CR>
nmap <silent> <Leader>c :call InstrumentHandle("cargo check --package scratch --features scratch/scrape")<CR>
nmap <silent> <Leader>C :call InstrumentHandle("cargo build --package plant")<CR>
nmap <silent> <Leader>B :call InstrumentHandle("cargo run -- browse")<CR>

nmap <silent> <C-s> :call InstrumentHandle("./shelf/web/start")<CR>

function! InstrumentCargo(...) abort
    "call setline('.', curline . ' ' . name)
    
    let get_path = 'cargo run --features magazine -- manifest-path ' . expand('%')
    let path = system(get_path)

    if path == ""
        throw "not a rust crate"
    endif

    call inputsave()
    let crates = input('crates: ')
    call inputrestore()

    if a:1 == "a"
        call feedkeys(':Cargo add --manifest-path ' . path . ' ' . crates)
    endif
endfunction

nmap <silent> <Leader>ca :call InstrumentCargo("a")<CR>

autocmd BufNewFile,BufRead * if expand('%:t') !~ '\.' | set spell | endif

if executable('rust-analyzer')
  au User lsp_setup call lsp#register_server({
        \   'name': 'Rust Language Server',
        \   'cmd': {server_info->['rust-analyzer']},
        \   'whitelist': ['rust'],
        \   'initialization_options': {
        \     'cargo': {
        \       'loadOutDirsFromCheck': v:true,
        \     },
        \     'procMacro': {
        \       'enable': v:true,
        \     },
        \   },
        \ })
endif

let g:lsp_diagnostics_echo_delay = 500
let g:lsp_diagnostics_enabled = 1
let g:lsp_diagnostics_echo_cursor = 1
let g:lsp_format_sync_timeout = 1000

function! s:on_lsp_buffer_enabled() abort
    setlocal omnifunc=lsp#complete
    setlocal signcolumn=yes
    nmap <buffer> <Leader>gd <plug>(lsp-definition)
    nmap <buffer> <Leader>gs <plug>(lsp-document-symbol-search)
    nmap <buffer> <Leader>gS <plug>(lsp-workspace-symbol-search)
    nmap <buffer> <Leader>gr <plug>(lsp-references)
    nmap <buffer> <Leader>gi <plug>(lsp-implementation)
    nmap <buffer> <Leader>gt <plug>(lsp-type-definition)
    nmap <buffer> <Leader>gn <plug>(lsp-rename)
    nmap <buffer> K <plug>(lsp-hover)
    inoremap <buffer> <expr><c-f> lsp#scroll(+4)
    inoremap <buffer> <expr><c-d> lsp#scroll(-4)

    autocmd! BufWritePre *.rs call execute('LspDocumentFormatSync')

    " refer to doc to add more commands
endfunction

augroup lsp_install
    au!
    " call s:on_lsp_buffer_enabled only for languages that has the server registered.
    autocmd User lsp_buffer_enabled call s:on_lsp_buffer_enabled()
augroup END

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

nmap <silent> <Leader>p :call InstrumentHandle("./shelf/web/start")<CR>
nmap <silent> <Leader>t :call InstrumentHandle("cargo test --package scratch --features scratch/search")<CR>
nmap <silent> <Leader>T :call InstrumentHandle("cargo test --package scratch --features scratch/search -- --nocapture")<CR>
nmap <silent> <Leader>c :call InstrumentHandle("cargo check --package scratch --features scratch/scrape")<CR>
nmap <silent> <Leader>C :call InstrumentHandle("cargo build --package plant")<CR>

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

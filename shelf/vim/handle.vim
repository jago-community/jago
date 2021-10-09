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

function! InstrumentHandle(...) abort
    if has('terminal')
        let cmd = 'vertical terminal'
    else
        throw "upgrade vim to version 8.1 or higher"
    endif

    let input = substitute(a:1, '%', expand('%'), '')

    let output = system('cargo run --features shell shell expand ' . input)
    let lines = split(output, "\n")
    let expanded = get(lines, len(lines) - 1)

    execute cmd expanded
endfunction

nmap <silent> <Leader>s :call InstrumentHandle("cargo run --features context,server serve")<CR>

nmap <silent> <Leader>t :call InstrumentHandle("cargo test {--package:%}")<CR>
nmap <silent> <Leader>tt :call InstrumentHandle("cargo test {--package:%} -- --nocapture")<CR>
nmap <silent> <Leader>wt :call InstrumentHandle("cargo test --workspace")<CR>
nmap <silent> <Leader>wtt :call InstrumentHandle("cargo test --workspace -- --nocapture")<CR>

nmap <silent> <Leader>jb :call InstrumentHandle("cargo build {package:%}")<CR>
nmap <silent> <Leader>jwb :call InstrumentHandle("cargo build --workspace")<CR>

nmap <silent> <Leader>ch :call InstrumentHandle("cargo build {package:%}")<CR>
nmap <silent> <Leader>wch :call InstrumentHandle("cargo build --workspace")<CR>

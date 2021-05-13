function! Handle(...) abort
    let kind = substitute(a:1, '\s\+$', '', '')

    if kind == "open"
        execute ':silent !open http://localhost:1342'
        redraw!
    else
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        if a:0 == 2 && a:2 == 1
            call inputsave()
            let rest = input('Rest: ')
            call inputrestore()
        endif

        let args = kind .' "'. rest . '"'

        execute cmd 'cargo' args
    endif
endfunction

nmap <silent> <Leader>c :call Handle("check")<CR>
nmap <silent> <Leader>C :call Handle("build")<CR>
nmap <silent> <Leader>t :call Handle("test")<CR>
nmap <silent> <Leader>T :call Handle("test -- --nocapture")<CR>
nmap <silent> <Leader>r :call Handle("run", 1)<CR>
nmap <silent> <Leader>rs :call Handle("serve")<CR>
nmap <silent> <Leader>g :call Handle("open")<CR>

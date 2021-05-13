function! s:Handle(...) abort
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

        let input = []

        for index in range(2, a:0)
            call add(input, a:000[index-1])
        endfor

        let rest = len(input) > 0 ? ' "' . join(input, ' ') . '"' : ''

        let args = kind . rest

        execute cmd 'cargo' args
    endif
endfunction

nmap <silent> <Leader>c :call s:Handle("check")<CR>
nmap <silent> <Leader>C :call s:Handle("build")<CR>
nmap <silent> <Leader>t :call s:Handle("test")<CR>
nmap <silent> <Leader>T :call s:Handle("test -- --nocapture")<CR>
nmap <silent> <Leader>r :call s:Handle("serve")<CR>
nmap <silent> <Leader>g :call s:Handle("open")<CR>

command -nargs=? J :call s:Handle("run", <f-args>)

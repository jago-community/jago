function! JHandle(...) abort
    let kind = substitute(a:1, '\s\+$', '', '')

    if kind == "container-delete" || kind == "container-build"
        call s:ContainerDelete("jago-serve")
    endif

    if kind == "open"
        execute ':silent !open http://localhost:1342'
        redraw!
    elseif kind == "container-build"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        let container = expand('$HOME') . '/local/jago/container/serve.Dockerfile'

        execute cmd 'docker build --tag jago-serve -f ' . container . ' .'
    elseif kind == "container-serve"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        execute cmd 'docker run --publish 1342:1342 --name jago-serve jago-serve'
    elseif kind == "container-delete"
        " nothing
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

function! s:ContainerDelete(args) abort
    " Trim trailing spaces. This is necessary since :terminal command parses
    " trailing spaces as an empty argument.
    let container = substitute(a:args, '\s\+$', '', '')

    let status = system('docker container inspect -f "{{.State.Status}}" ' . container)

    let status = substitute(status, '\s\+$', '', '')
    let status = substitute(status, '\n\+$', '', '')

    " echo status

    if status == "running"
        let _stop = system('docker stop ' . container)
        "echo _stop
    endif

    if status == "created" || status == "running" || status == "exited"
        let _removed = system('docker rm ' . container)
        "echo _removed
    endif
endfunction

nmap <silent> <Leader>c :call JHandle("check")<CR>
nmap <silent> <Leader>C :call JHandle("build")<CR>
nmap <silent> <Leader>t :call JHandle("test")<CR>
nmap <silent> <Leader>T :call JHandle("test -- --nocapture")<CR>
nmap <silent> <Leader>r :call JHandle("run", "serve")<CR>
nmap <silent> <Leader>g :call JHandle("open")<CR>

nmap <silent> <Leader>db :call JHandle("container-build")<CR>
nmap <silent> <Leader>ds :call JHandle("container-serve")<CR>
nmap <silent> <Leader>dd :call JHandle("container-delete")<CR>

command -nargs=? J :call JHandle("run", <f-args>)

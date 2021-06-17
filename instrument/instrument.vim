function! JHandle(...) abort
    let kind = substitute(a:1, '\s\+$', '', '')

    if kind == "open"
        execute ':silent !open http://localhost:1342'
        redraw!
    elseif kind == "container-deploy"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        execute cmd 'kubectl apply -f stack/deployment.yml'
    elseif kind == "container-push"
        let commit_hash = system('git rev-parse --short HEAD')
        let commit_hash = substitute(commit_hash, '\_s*$', '', '')
        call setreg("v", commit_hash)

        let tag = 'gcr.io/jago-277604/jago:' . commit_hash

        let _res = system('docker tag jago-serve ' . tag)

        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        execute cmd 'docker push ' . tag
    elseif kind == "container-build"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        let scripts = expand('$HOME') . '/local/jago/instrument/container/'

        let cmd = cmd . ' ' . scripts . 'build'

        execute cmd
    elseif kind == "container-serve"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        let scripts = expand('$HOME') . '/local/jago/instrument/container/'

        let append = a:2 == 1 ? ' 1' : ' 0'

        let cmd = cmd . ' ' . scripts . 'start' . append

        execute cmd
    elseif kind == "container-logs"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        let compose = expand('$HOME') . '/local/jago/container/compose.yml'

        execute cmd 'docker compose -f ' . compose . ' logs'
    elseif kind == "container-delete"
        if has('terminal')
            let cmd = 'vertical terminal'
        else
            throw "upgrade vim to version 8.1 or higher"
        endif

        let scripts = expand('$HOME') . '/local/jago/instrument/container/'

        execute cmd . ' ' . scripts . 'clean'
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

        let rest = len(input) > 0 ? ' ' . join(input, ' ') : ''

        let args = kind . rest

        execute cmd 'cargo' args
    endif
endfunction

nmap <silent> <Leader>c :call JHandle("check")<CR>
nmap <silent> <Leader>C :call JHandle("build")<CR>
nmap <silent> <Leader>t :call JHandle("test")<CR>
nmap <silent> <Leader>T :call JHandle("test -- --nocapture")<CR>
nmap <silent> <Leader>r :call JHandle("run", "serve")<CR>
nmap <silent> <Leader>g :call JHandle("open")<CR>

nmap <silent> <Leader>db :call JHandle("container-build")<CR>
nmap <silent> <Leader>ds :call JHandle("container-serve", 0)<CR>
nmap <silent> <Leader>dS :call JHandle("container-serve", 1)<CR>
nmap <silent> <Leader>dp :call JHandle("container-push")<CR>
nmap <silent> <Leader>dd :call JHandle("container-delete")<CR>
nmap <silent> <Leader>du :call JHandle("container-deploy")<CR>
nmap <silent> <Leader>dl :call JHandle("container-logs")<CR>

nnoremap <Leader>? :echo "Hello, world!"<CR>

command -nargs=? J :call JHandle("run", <f-args>)

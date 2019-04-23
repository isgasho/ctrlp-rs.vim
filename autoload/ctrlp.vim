" Vim script for interaction for ctrlp

" Default job id of 0 (no running job)
if ! exists('s:job_id')
    let s:job_id = 0
endif

let s:bin = resolve(expand('<sfile>:p:h') . '/../target/release/ctrlp')


" Initialize ctrlp plugin
function! ctrlp#init()
    let result = s:startup()

    call rpcnotify(s:job_id, 'startup')
    if 0 == result
        echoerr "ctrlp: Failed to start process"
    elseif -1 == result
        echoerr "ctrlp: Binary not executable"
    else
        let s:job_id = result
        call s:ConfigureJob(result)
    endif
endfunction


" Startup/shutdown
function s:shutdown()
    call rpcnotify(s:job_id, 'shutdown')
endfunction


function! s:startup()
    if 0 == s:job_id
        let id = jobstart([s:bin], { 'rpc': v:true, 'on_stderr': function('s:OnStderr') })
        return id
    else
        return 0
    endif
endfunction

function! s:shutdown()
    if 0 > s:job_id
        augroup ctrlp
            " clear all previous autocommands
            autocmd!
        augroup END

        " Give the job half a second to stop by itself
        call rpcnotify(s:job_id, 'shutdown')
        let result = jobwait(s:job_id, 500)

        " If it still didn't shut down properly, kill it
        if -1 == result
            call jobstop(s:job_id)
        endif

        " Reset the job id
        let s:job_id = 0
    endif
endfunction


" Configuration
function! s:ConfigureJob(job_id)
    augroup ctrlp
        " clear all previous autocommands
        autocmd!

        autocmd VimLeavePre * :call s:shutdown()
    augroup END
endfunction


" Event handler
function! s:NotifyInsertLeave()
    call rpcnotify(s:job_id, 'lol')
endfunction

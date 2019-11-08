if exists('g:discuss_code#loaded')
	finish
endif

hi annotatedLine ctermbg=blue guibg=blue
sign define annotation text=ﳴ texthl=annotatedLine linehl=annotatedLine
sign define annotationContinued text= texthl=annotatedLine linehl=annotatedLine

if !exists('s:discuss_job_id')
	let s:discuss_job_id = 0
endif

let s:bin = expand('<sfile>:p:h:h') . '/target/release/discuss-code'

function! s:connect()
	let id = s:init_rpc()

	if 0 == id
		echoerr "[discuss-code]: Cannot start RPC process"
	elseif -1 == id
		echoerr "[discuss-code]: RPC process is not executable"
	else
		let s:discuss_job_id = id

		call s:configure_commands()
	endif
endfunction

function! s:configure_commands()
	vnoremap ac :call discuss_code#New_comment(@%)<cr>
	nnoremap sc :call discuss_code#Show_comment(@%)<cr>
	nnoremap dc :call discuss_code#Delete_comment(@%)<cr>
endfunction

let s:NewComment = 'new_comment'
let s:DeleteComment = 'delete_comment'
let s:ShowComment = 'show_comment'
let s:HighlightComments = 'highlight_comments'

function! discuss_code#New_comment(file_name) range
	call inputsave()
	let l:comment = input('Comment: ')
	call inputrestore()
	call rpcnotify(s:discuss_job_id, s:NewComment, a:file_name, a:firstline, a:lastline, l:comment)
endfunction

function! discuss_code#Highlight_comments(file_name)
	call rpcnotify(s:discuss_job_id, s:HighlightComments, a:file_name)
endfunction

function! discuss_code#Delete_comment(file_name) range
	call rpcnotify(s:discuss_job_id, s:DeleteComment, a:file_name, a:firstline)
endfunction

function! discuss_code#Show_comment(file_name) range
	call rpcnotify(s:discuss_job_id, s:ShowComment, a:file_name, a:firstline)
endfunction

function! discuss_code#Display_comment(comment)
	call discuss_code#Hide_comments()
	let buf = nvim_create_buf(v:false, v:true)
	call nvim_buf_set_lines(buf, 0, -1, v:true, ["", " " . a:comment])
	let opts = {'relative': 'cursor', 'width': strlen(a:comment) + 2, 'height': 3, 'col': 0,
				\ 'row': 1, 'anchor': 'NW', 'style': 'minimal'}
	let g:discuss_code#comment_win = nvim_open_win(buf, 0, opts)
endfunction

function! discuss_code#Hide_comments()
	if exists('g:discuss_code#comment_win')
		let id = win_id2win(g:discuss_code#comment_win)
		if id > 0
			execute id . 'close!'
		endif
	endif
endfunction

function! s:init_rpc()
	if s:discuss_job_id == 0
		let job_id = jobstart([s:bin], { 'rpc': v:true })
		return job_id
	else
		return s:discuss_job_id
	endif
endfunction

call s:connect()

augroup init_buffer
	au!
	au BufWinEnter * call discuss_code#Highlight_comments(@%)
	au CursorMoved * call discuss_code#Hide_comments()
augroup END

let g:discuss_code#loaded = 1

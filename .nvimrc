command! RunCargo silent execute '!(kitty sh -c "cargo run || read" &)'
nnoremap <F5> <cmd>RunCargo<CR>

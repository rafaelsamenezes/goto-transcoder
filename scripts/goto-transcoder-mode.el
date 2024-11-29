;;; goto_transcoder-mode.el --- Defines the elisp functions for this project -*- lexical-binding: t -*-
;;; Commentary:
;;; Code:

(defun goto-transcoder-setup ()
  "Set up the project main commands for use with project."
  (setq-local compile-command "cargo build"))

(defun goto-transcoder-test ()
  "Run all test cases from the project."
  (interactive)
  (async-shell-command "cargo test -- --ignored"))

(define-minor-mode goto-transcoder-mode
  "A minor mode for the goto-transcoder features."
  :lighter " GotoTranscoder"
  :keymap (let ((map (make-sparse-keymap)))
            ;; Define keybindings here
            (define-key map (kbd "C-x p t") 'goto-transcoder-test)
            map)
  ;; Code to run when the mode is activated
  (if goto-transcoder-mode (goto-transcoder-setup)
    ()))

(provide 'goto-transcoder-mode)
;;; goto-transcoder-mode.el ends here

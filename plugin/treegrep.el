;;; treegrep.el --- Run tgrep in a terminal window -*- lexical-binding: t -*-

;; Copyright (C) 2026 Timothy Cronin
;;
;; Author: Timothy Cronin <tcroniniv@gmail.com>
;; URL: https://github.com/4imothy/treegrep
;; SPDX-License-Identifier: MIT

;;; Commentary:

;; Run tgrep in a terminal buffer and open the selected file on exit.
;; Only --selection-file and --repeat-file are added automatically.
;;
;; Required: tgrep-selection-file, tgrep-terminal (returns the process).
;; Optional: tgrep-repeat-file, tgrep-binary.
;;
;; tgrep-terminal receives a shell command string and must display a
;; terminal buffer, run the command interactively, and return the process.

;;; Code:

(defgroup tgrep nil
  "Run tgrep in a terminal window."
  :group 'tools)

(defcustom tgrep-terminal nil
  "Function to launch tgrep.  Called with a shell command string, must return the process."
  :type 'function
  :group 'tgrep)

(defcustom tgrep-selection-file nil
  "File tgrep writes its selection to (path on line 1, line number on line 2)."
  :type 'string
  :group 'tgrep)

(defcustom tgrep-repeat-file nil
  "File tgrep uses to save and replay searches.  Optional."
  :type '(choice (const nil) string)
  :group 'tgrep)

(defcustom tgrep-binary nil
  "Path to the tgrep binary.  When nil, resolved relative to this file's directory."
  :type '(choice (const nil) string)
  :group 'tgrep)

(defvar tgrep--dir
  (when-let ((f (or (locate-library "treegrep") load-file-name)))
    (file-name-directory (file-truename f))))

(defun tgrep--binary ()
  (or tgrep-binary
      (expand-file-name "../target/release/tgrep" tgrep--dir)))

(defun tgrep--build-command (args)
  (concat (shell-quote-argument (tgrep--binary))
          " --no-alternate-screen"
          " --selection-file=" (shell-quote-argument tgrep-selection-file)
          (when tgrep-repeat-file
            (concat " --repeat-file=" (shell-quote-argument tgrep-repeat-file)))
          (when (and args (not (string-empty-p args)))
            (concat " " args))))

(defun tgrep--on-exit (proc _event)
  (when (memq (process-status proc) '(exit signal))
    (let ((window-config (process-get proc 'tgrep-window-config)))
      (when (buffer-live-p (process-buffer proc))
        (kill-buffer (process-buffer proc)))
      (when window-config
        (set-window-configuration window-config))
      (when (and tgrep-selection-file
                 (file-readable-p tgrep-selection-file))
        (let ((lines (with-temp-buffer
                       (insert-file-contents tgrep-selection-file)
                       (split-string (buffer-string) "\n" t))))
          (when lines
            (find-file (car lines))
            (when (cadr lines)
              (goto-char (point-min))
              (forward-line
               (1- (string-to-number (cadr lines)))))))))))

;;;###autoload
(defun tgrep-with (args)
  "Run tgrep with ARGS in a terminal window."
  (unless tgrep-selection-file
    (user-error "treegrep: set `tgrep-selection-file' before calling `tgrep-with'"))
  (unless tgrep-terminal
    (user-error "treegrep: set `tgrep-terminal' before calling `tgrep-with'"))
  (let* ((window-config (current-window-configuration))
         (proc (funcall tgrep-terminal (tgrep--build-command args))))
    (process-put proc 'tgrep-window-config window-config)
    (set-process-sentinel proc #'tgrep--on-exit)))

;;;###autoload
(defun tgrep-build ()
  "Build the tgrep binary from source using cargo."
  (interactive)
  (let* ((manifest (expand-file-name "../Cargo.toml" tgrep--dir))
         (cmd (format "cargo build --release --color never --manifest-path=%s"
                      (shell-quote-argument manifest)))
         (buffer (get-buffer-create "*tgrep-build*")))
    (with-current-buffer buffer (erase-buffer))
    (message "treegrep: building tgrep...")
    (set-process-sentinel
      (start-process-shell-command "tgrep-build" buffer cmd)
      (lambda (proc _event)
        (when (memq (process-status proc) '(exit signal))
          (if (eq (process-exit-status proc) 0)
              (message "treegrep: tgrep built")
            (display-buffer (process-buffer proc))
            (message "treegrep: tgrep build failed; see *tgrep-build*")))))
    nil))

(provide 'treegrep)

;;; treegrep.el ends here

// Client-side validation called by ic-on-beforeSend
// Intercooler's globalEval passes: elt, data, settings, xhr
function validateForm() {
    var $error = $('#error-message');
    var method = $('input[name="restore-method"]:checked').val();

    if (method === 'recovery-key') {
        var allFilled = true;
        $('.cd-key-block').each(function() {
            if ($(this).val().length < parseInt($(this).attr('maxlength'))) {
                allFilled = false;
                return false;
            }
        });
        if (!allFilled) {
            $error.text('Please fill in all recovery key fields.').show();
            return false;
        }
    } else if (method === 'passphrase') {
        var passphrase = $('#passphrase').val().trim();
        if (!passphrase) {
            $error.text('Please enter your passphrase.').show();
            return false;
        }
    }

    $error.hide();
    return true;
}

function initVerify(rootContent) {
    // If no rootContent provided (from $(document).ready), use $(document)
    var $root = rootContent || $(document);

    var blocks = $root.find('.cd-key-block');
    var totalBlocks = blocks.length;

    // Only initialize if we have blocks
    if (blocks.length === 0) {
        return;
    }

    // Update option card selection
    function updateOptionSelection(selectedValue) {
        $root.find('.clickable-option').removeClass('selected');
        if (selectedValue === 'recovery-key') {
            $root.find('#card-recovery-key').addClass('selected');
        } else if (selectedValue === 'passphrase') {
            $root.find('#card-passphrase').addClass('selected');
        }
    }

    // Update the hidden passphrase field
    function updateHiddenField() {
        var values = [];
        blocks.each(function() {
            values.push($(this).val());
        });
        $root.find('#recovery_key_full').val(values.join(""));
    }

    function getClipboardData(e) {
        if (window.clipboardData && window.clipboardData.getData) {
            return window.clipboardData.getData('Text');
        } else if (e.originalEvent && e.originalEvent.clipboardData) {
            return e.originalEvent.clipboardData.getData('text');
        } else if (e.clipboardData) {
            return e.clipboardData.getData('text');
        }
        return null;
    }

    function focusAndMoveCursorToEnd(input) {
        input.focus();
        if (input[0].createTextRange) {
            var range = input[0].createTextRange();
            range.collapse(false);
            range.select();
        } else if (input[0].setSelectionRange) {
            var len = input.val().length;
            input[0].setSelectionRange(len, len);
        }
    }

    function resetErrors() {
        $root.find('#error-message').hide();
    }

    // Clear error when user starts typing
    blocks.on('input', function() {
        resetErrors()
    });
    $root.find('#passphrase').on('input', function() {
        resetErrors()
    });

    // Initialize
    updateHiddenField();
    updateOptionSelection('recovery-key');

    // Handle restore method toggle
    $root.find('input[name="restore-method"]').on('click', function() {
        var method = $(this).val();
        updateOptionSelection(method);

        if (method === 'recovery-key') {
            $root.find('#recovery-key-section').show();
            $root.find('#passphrase-section').hide();
        } else if (method === 'passphrase') {
            $root.find('#recovery-key-section').hide();
            $root.find('#passphrase-section').show();
        }
    });

    $root.find('.clickable-option').on('click', function(e) {
        if (e.target.type === 'radio') {
            return;
        }

        resetErrors()

        var radio = $(this).find('input[type="radio"]');
        var method = radio.val();
        radio.prop('checked', true);
        updateOptionSelection(method);

        if (method === 'recovery-key') {
            $root.find('#recovery-key-section').show();
            $root.find('#passphrase-section').hide();
        } else if (method === 'passphrase') {
            $root.find('#recovery-key-section').hide();
            $root.find('#passphrase-section').show();
        }
    });

    // Auto-advance on input
    blocks.on('input', function() {
        var currentIndex = parseInt($(this).attr('data-index'));
        var value = $(this).val();

        if (value.length === parseInt($(this).attr('maxlength'))) {
            if (currentIndex < totalBlocks - 1) {
                blocks.eq(currentIndex + 1).focus();
            }
        }
        updateHiddenField();
    });

    // Handle backspace to go to previous block
    blocks.on('keydown', function(e) {
        var currentIndex = parseInt($(this).attr('data-index'));
        if (e.keyCode === 8 && $(this).val().length === 0 && currentIndex > 0) {
            focusAndMoveCursorToEnd(blocks.eq(currentIndex - 1));
        }
    });

    // Handle paste across all blocks
    blocks.on('paste', function(e) {
        e.preventDefault();
        var pasteData = getClipboardData(e);
        if (!pasteData) return;

        var cleanData = pasteData.replace(/[\s\-]/g, '');
        var blockSize = parseInt($(this).attr('maxlength'));

        blocks.each(function(index) {
            var start = index * blockSize;
            var chunk = cleanData.substring(start, start + blockSize);
            $(this).val(chunk);
        });

        updateHiddenField();
        var lastFilledIndex = Math.min(Math.floor(cleanData.length / blockSize), totalBlocks - 1);
        blocks.eq(lastFilledIndex).focus();
    });
}

// Initialize on document ready (for initial page load)
$(document).ready(function() {
    initVerify();
});

// Initialize on Intercooler content swap
Intercooler.ready(function(rootContent) {
    initVerify(rootContent);
});
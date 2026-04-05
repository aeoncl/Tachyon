$(document).ready(function() {
    var url = new URL(window.location);
    if (url.searchParams.has('t')) {
        url.searchParams.delete('t');
        window.history.replaceState({}, '', url);
    }
});
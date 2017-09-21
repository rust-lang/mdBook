(function() {
// register a function run when the page is loaded completely
document.addEventListener('DOMContentLoaded', function() {
    // we extend all elements with class="custom_border" with a little text
    var divs = document.querySelectorAll('.custom_border');
    divs.forEach(function(e) {
        try {
            e.innerHTML += ' - extended with JavaScript';
        } catch (e) {}
    });
}, false,);
})();

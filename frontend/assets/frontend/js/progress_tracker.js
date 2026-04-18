(function() {
  // Try to detect chapter heading - look for h1, h2, h3
  function detectChapter() {
    var headings = document.querySelectorAll('h1, h2, h3');
    for (var i = 0; i < headings.length; i++) {
      var text = headings[i].textContent.trim();
      if (text.length > 0 && text.length < 200) return text;
    }
    return null;
  }

  // Calculate scroll progress
  function scrollProgress() {
    var scrollTop = window.scrollY || document.documentElement.scrollTop;
    var docHeight = document.documentElement.scrollHeight - document.documentElement.clientHeight;
    if (docHeight <= 0) return 1.0;
    return Math.min(1.0, scrollTop / docHeight);
  }

  function sendUpdate() {
    var chapter = detectChapter();
    var progress = scrollProgress();
    ProgressSignal.postMessage(JSON.stringify({
      chapter: chapter,
      progress: progress
    }));
  }

  // Send on load and on scroll (throttled)
  sendUpdate();
  var lastSent = Date.now();
  window.addEventListener('scroll', function() {
    var now = Date.now();
    if (now - lastSent > 500) {
      lastSent = now;
      sendUpdate();
    }
  });
})();

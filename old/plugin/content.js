// Content script for Video Download Assistant

class VideoInfoExtractor {
  constructor() {
    this.videoInfo = null;
    this.init();
  }

  init() {
    // Extract video information on page load
    this.extractVideoInfo();
    
    // Re-extract if URL changes (for SPAs)
    let currentUrl = window.location.href;
    const observer = new MutationObserver(() => {
      if (window.location.href !== currentUrl) {
        currentUrl = window.location.href;
        setTimeout(() => this.extractVideoInfo(), 1000);
      }
    });
    
    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }

  extractVideoInfo() {
    const url = window.location.href;
    let title = '';
    let thumbnail = '';

    // Try to extract title from various sources
    const titleSelectors = [
      'title',
      'h1',
      '[data-title]',
      '.video-title',
      '.title',
      'meta[property="og:title"]',
      'meta[name="title"]'
    ];

    for (const selector of titleSelectors) {
      const element = document.querySelector(selector);
      if (element) {
        if (element.tagName === 'META') {
          title = element.content;
        } else {
          title = element.textContent || element.innerText;
        }
        if (title && title.trim()) {
          title = title.trim();
          break;
        }
      }
    }

    // Try to extract thumbnail
    const thumbnailSelectors = [
      'meta[property="og:image"]',
      'meta[name="twitter:image"]',
      'video',
      '.video-thumbnail img',
      '.thumbnail img'
    ];

    for (const selector of thumbnailSelectors) {
      const element = document.querySelector(selector);
      if (element) {
        if (element.tagName === 'META') {
          thumbnail = element.content;
        } else if (element.tagName === 'VIDEO') {
          thumbnail = element.poster || '';
        } else {
          thumbnail = element.src || '';
        }
        if (thumbnail) break;
      }
    }

    // Clean title
    if (title) {
      title = title.replace(/^\s*-\s*/, '').replace(/\s*-\s*$/, '');
      title = title.replace(/\s+/g, ' ').trim();
      if (title.length > 100) {
        title = title.substring(0, 100) + '...';
      }
    }

    this.videoInfo = {
      url: url,
      title: title || 'Unknown Title',
      thumbnail: thumbnail,
      timestamp: Date.now()
    };

    // Send to background script
    chrome.runtime.sendMessage({
      action: 'getVideoInfo',
      data: this.videoInfo
    });
  }

  getVideoInfo() {
    return this.videoInfo;
  }
}

// Initialize video info extractor
const videoExtractor = new VideoInfoExtractor();

// Listen for messages from popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'getPageVideoInfo') {
    sendResponse(videoExtractor.getVideoInfo());
  }
});
// Dialog script for Video Download Assistant

class DownloadDialog {
  constructor() {
    this.videoInfo = null;
    this.settings = null;
    this.translations = {};
    this.init();
  }

  async init() {
    // Get video info from URL params
    const params = new URLSearchParams(window.location.search);
    const videoInfoJson = params.get('videoInfo');
    
    if (videoInfoJson) {
      this.videoInfo = JSON.parse(decodeURIComponent(videoInfoJson));
    }
    
    // Load settings and translations
    await this.loadSettings();
    await this.loadTranslations();
    
    // Initialize UI
    this.initializeElements();
    this.attachEventListeners();
    this.updateVideoInfo();
  }

  initializeElements() {
    this.dialogThumbnail = document.getElementById('dialog-thumbnail');
    this.dialogThumbnailPlaceholder = document.getElementById('dialog-thumbnail-placeholder');
    this.dialogTitle = document.getElementById('dialog-title');
    this.downloadTypeRadios = document.querySelectorAll('input[name="downloadType"]');
    this.videoQualityGroup = document.getElementById('video-quality-group');
    this.videoQualitySelect = document.getElementById('video-quality-select');
    this.audioQualitySelect = document.getElementById('audio-quality-select');
    this.audioFormatSelect = document.getElementById('audio-format-select');
    this.dialogDownloadBtn = document.getElementById('dialog-download-btn');
    this.dialogDownloadText = document.getElementById('dialog-download-text');
    this.dialogSpinner = document.getElementById('dialog-spinner');
    this.dialogStatus = document.getElementById('dialog-status');
    this.dialogStatusText = document.getElementById('dialog-status-text');
  }

  attachEventListeners() {
    // Download type change
    this.downloadTypeRadios.forEach(radio => {
      radio.addEventListener('change', (e) => this.onDownloadTypeChange(e.target.value));
    });
    
    // Download button
    this.dialogDownloadBtn.addEventListener('click', () => this.startDownload());
  }

  async loadSettings() {
    const stored = await chrome.storage.sync.get(['language', 'serverPort', 'downloadFolder']);
    this.settings = {
      language: stored.language || 'en',
      serverPort: stored.serverPort || 8080,
      downloadFolder: stored.downloadFolder || 'Downloads'
    };
  }

  async loadTranslations() {
    try {
      const response = await fetch(`/_locales/${this.settings.language}/messages.json`);
      const messages = await response.json();
      
      this.translations = {};
      for (const [key, value] of Object.entries(messages)) {
        this.translations[key] = value.message;
      }
      
      this.updateUIText();
    } catch (error) {
      console.error('Failed to load translations:', error);
    }
  }

  updateUIText() {
    document.getElementById('download-type-label').textContent = this.translations.downloadType || 'Download Type:';
    document.getElementById('download-video-label').textContent = this.translations.downloadVideo || 'Video';
    document.getElementById('download-audio-label').textContent = this.translations.downloadAudioOnly || 'Audio Only';
    document.getElementById('video-quality-label').textContent = this.translations.videoQuality || 'Video Quality:';
    document.getElementById('audio-quality-label').textContent = this.translations.audioQuality || 'Audio Quality:';
    document.getElementById('audio-format-label').textContent = this.translations.audioFormat || 'Audio Format:';
    this.dialogDownloadText.textContent = this.translations.download || 'Download';
    
    // Update quality options
    const qualityOptions = this.videoQualitySelect.options;
    qualityOptions[0].textContent = this.translations.bestQuality || 'Best Quality (1440p/1080p)';
    qualityOptions[1].textContent = this.translations.quality1080p || '1080p or lower';
    qualityOptions[2].textContent = this.translations.hd720Quality || '720p or lower';
    qualityOptions[3].textContent = this.translations.sd480Quality || '480p or lower';
    qualityOptions[4].textContent = this.translations.lowestQuality || 'Lowest Quality';
  }

  updateVideoInfo() {
    if (!this.videoInfo) return;
    
    this.dialogTitle.textContent = this.videoInfo.title || 'Unknown Title';
    
    if (this.videoInfo.thumbnail) {
      this.dialogThumbnail.src = this.videoInfo.thumbnail;
      this.dialogThumbnail.style.display = 'block';
      this.dialogThumbnailPlaceholder.style.display = 'none';
    } else {
      this.dialogThumbnail.style.display = 'none';
      this.dialogThumbnailPlaceholder.style.display = 'flex';
    }
  }

  onDownloadTypeChange(type) {
    if (type === 'video') {
      this.videoQualityGroup.style.display = 'block';
    } else {
      this.videoQualityGroup.style.display = 'none';
    }
  }

  async startDownload() {
    if (!this.videoInfo) return;
    
    // Disable button
    this.dialogDownloadBtn.disabled = true;
    this.dialogDownloadText.style.display = 'none';
    this.dialogSpinner.style.display = 'block';
    this.dialogStatus.style.display = 'none';
    
    try {
      // Get download options
      const downloadType = document.querySelector('input[name="downloadType"]:checked').value;
      const videoQuality = this.videoQualitySelect.value;
      const audioQuality = this.audioQualitySelect.value;
      const audioFormat = this.audioFormatSelect.value;
      
      // Prepare download data
      const downloadData = {
        url: this.videoInfo.url,
        title: this.videoInfo.title,
        downloadType: downloadType,
        videoQuality: videoQuality,
        audioQuality: audioQuality,
        audioFormat: audioFormat,
        folder: this.settings.downloadFolder
      };
      
      // Send to server
      const response = await fetch(`http://localhost:${this.settings.serverPort}/download`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(downloadData)
      });
      
      if (response.ok) {
        const result = await response.json();
        this.showStatus(this.translations.downloadComplete || 'Download started! Check your downloads folder.', 'success');
        
        // Close window after 2 seconds
        setTimeout(() => {
          window.close();
        }, 2000);
      } else {
        throw new Error('Server error');
      }
    } catch (error) {
      console.error('Download failed:', error);
      this.showStatus(this.translations.downloadFailed || 'Download failed. Please check server connection.', 'error');
    } finally {
      // Re-enable button
      this.dialogDownloadBtn.disabled = false;
      this.dialogDownloadText.style.display = 'inline';
      this.dialogSpinner.style.display = 'none';
    }
  }

  showStatus(message, type) {
    this.dialogStatusText.textContent = message;
    this.dialogStatus.className = 'dialog-status ' + type;
    this.dialogStatus.style.display = 'block';
  }
}

// Initialize the dialog
document.addEventListener('DOMContentLoaded', () => {
  new DownloadDialog();
});
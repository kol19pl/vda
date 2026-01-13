export function updateUI(vda) {
    const elements = document.querySelectorAll('[id]');
    elements.forEach(element => {
        const key = vda.getTranslationKey(element.id);
        if (key && vda.translations[key]) {
            if (element.tagName === 'INPUT' && element.type === 'button') {
                element.value = vda.t(key);
            } else if (element.tagName === 'BUTTON') {
                const iconElements = element.querySelectorAll('.spinner');
                if (iconElements.length === 0) {
                    element.textContent = vda.t(key);
                } else {
                    const textSpan = element.querySelector('span:not(.spinner)');
                    if (textSpan) textSpan.textContent = vda.t(key);
                }
            } else if (element.tagName === 'LABEL' || element.tagName === 'SPAN') {
                element.textContent = vda.t(key);
            }
        }
    });

    updateQualityOptions(vda);
    updateFormatOptions(vda);
    vda.updateSettingsFields();
}

export function updateQualityOptions(vda) {
    const qualitySelect = document.getElementById('quality-select');
    const options = [
        { value: 'best', key: 'bestQuality' },
        { value: 'best[height<=720]', key: 'hd720Quality' },
        { value: 'best[height<=480]', key: 'sd480Quality' },
        { value: 'worst', key: 'lowestQuality' },
        { value: 'bestaudio', key: 'audioOnly' }
    ];

    const currentValue = qualitySelect.value;
    qualitySelect.innerHTML = '';

    options.forEach(option => {
        const opt = document.createElement('option');
        opt.value = option.value;
        opt.textContent = vda.t(option.key) || vda.getDefaultQualityText(option.value);
        qualitySelect.appendChild(opt);
    });

    qualitySelect.value = currentValue;
}

export function updateFormatOptions(vda) {
    const formatSelect = document.getElementById('format-select');
    const options = [
        { value: 'mp4', text: 'MP4' },
        { value: 'mkv', text: 'MKV' },
        { value: 'webm', text: 'WebM' },
        { value: 'mp3', key: 'audioMp3' }
    ];

    const currentValue = formatSelect.value;
    formatSelect.innerHTML = '';

    options.forEach(option => {
        const opt = document.createElement('option');
        opt.value = option.value;
        opt.textContent = option.key ? vda.t(option.key) : option.text;
        formatSelect.appendChild(opt);
    });

    formatSelect.value = currentValue;
}

export function showStatus(vda, message, type) {
    const statusDiv = document.getElementById('download-status');
    const messageElement = document.getElementById('status-message-text');
    const messageDiv = document.querySelector('.status-message');

    messageElement.textContent = message;
    messageDiv.className = `status-message ${type}`;
    statusDiv.style.display = 'block';

    if (type === 'success') {
        setTimeout(() => { statusDiv.style.display = 'none'; }, 5000);
    }
}

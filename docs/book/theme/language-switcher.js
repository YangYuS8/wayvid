// Language Switcher
(function() {
    'use strict';
    
    // Language configuration
    const languages = {
        'en': { name: 'English', path: '' },
        'zh-CN': { name: '简体中文', path: '/zh_cn' }
    };
    
    // Detect current language from path
    function getCurrentLanguage() {
        const path = window.location.pathname;
        if (path.includes('/zh_cn/')) {
            return 'zh-CN';
        }
        return 'en';
    }
    
    // Get corresponding page in target language
    function getTranslatedPath(targetLang) {
        const currentPath = window.location.pathname;
        const currentLang = getCurrentLanguage();
        
        if (currentLang === targetLang) {
            return currentPath;
        }
        
        if (targetLang === 'zh-CN') {
            // English to Chinese
            if (currentPath.includes('/zh_cn/')) {
                return currentPath; // Already Chinese
            }
            // Insert zh_cn/ before the filename
            // /introduction.html -> /zh_cn/introduction.html
            // /user-guide/quick-start.html -> /zh_cn/user-guide/quick-start.html
            return currentPath.replace(/\/([^/]+\.html)$/, '/zh_cn/$1')
                              .replace(/\/(user-guide|features|dev|reference)\//, '/zh_cn/$1/');
        } else {
            // Chinese to English
            if (!currentPath.includes('/zh_cn/')) {
                return currentPath; // Already English
            }
            // Remove zh_cn/ from path
            return currentPath.replace('/zh_cn/', '/');
        }
    }
    
    // Create language switcher
    function createLanguageSwitcher() {
        const switcher = document.createElement('div');
        switcher.className = 'language-switcher';
        
        const select = document.createElement('select');
        select.setAttribute('aria-label', 'Select language');
        
        const currentLang = getCurrentLanguage();
        
        for (const [code, info] of Object.entries(languages)) {
            const option = document.createElement('option');
            option.value = code;
            option.textContent = info.name;
            if (code === currentLang) {
                option.selected = true;
            }
            select.appendChild(option);
        }
        
        select.addEventListener('change', function() {
            const targetLang = this.value;
            const newPath = getTranslatedPath(targetLang);
            window.location.href = newPath;
        });
        
        switcher.appendChild(select);
        document.body.appendChild(switcher);
    }
    
    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', createLanguageSwitcher);
    } else {
        createLanguageSwitcher();
    }
})();

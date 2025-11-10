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
        
        // Map of page correspondences
        const pageMap = {
            'introduction.html': 'zh_cn/introduction.html',
            'user-guide/quick-start.html': 'zh_cn/user-guide/quick-start.html',
            'user-guide/installation.html': 'zh_cn/user-guide/installation.html',
            'user-guide/configuration.html': 'zh_cn/user-guide/configuration.html',
            'user-guide/video-sources.html': 'zh_cn/user-guide/video-sources.html',
            'user-guide/multi-monitor.html': 'zh_cn/user-guide/multi-monitor.html',
            'features/hdr.html': 'zh_cn/features/hdr.html',
            'features/workshop.html': 'zh_cn/features/workshop.html',
            'features/niri.html': 'zh_cn/features/niri.html',
            'features/ipc.html': 'zh_cn/features/ipc.html',
            'dev/building.html': 'zh_cn/dev/building.html',
            'dev/workflow.html': 'zh_cn/dev/workflow.html',
            'dev/architecture.html': 'zh_cn/dev/architecture.html',
            'dev/contributing.html': 'zh_cn/dev/contributing.html',
            'reference/config.html': 'zh_cn/reference/config.html',
            'reference/cli.html': 'zh_cn/reference/cli.html',
            'reference/ipc-protocol.html': 'zh_cn/reference/ipc-protocol.html',
            'reference/we-format.html': 'zh_cn/reference/we-format.html'
        };
        
        // Extract relative page path
        let pagePath = currentPath.split('/book/')[1] || 'introduction.html';
        
        if (targetLang === 'zh-CN') {
            // Switch to Chinese
            if (currentLang === 'en') {
                return pageMap[pagePath] 
                    ? `/book/${pageMap[pagePath]}`
                    : '/book/zh_cn/introduction.html';
            }
        } else {
            // Switch to English
            if (currentLang === 'zh-CN') {
                // Reverse lookup
                const enPath = Object.keys(pageMap).find(
                    key => pageMap[key] === pagePath.replace('zh_cn/', '')
                );
                return enPath ? `/book/${enPath}` : '/book/introduction.html';
            }
        }
        
        return currentPath;
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

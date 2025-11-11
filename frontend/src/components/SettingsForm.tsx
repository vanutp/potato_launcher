import {useState, useEffect} from 'react';
import {Save} from 'lucide-react';

interface SettingsData {
    download_server_base: string;
    resources_url_base: string;
    replace_download_urls: boolean;
}

interface SettingsFormProps {
    onSave: (settings: SettingsData) => void;
}

export default function SettingsForm({onSave}: SettingsFormProps) {
    const [settings, setSettings] = useState<SettingsData>({
        download_server_base: 'https://download.minecraft.net',
        resources_url_base: 'https://resources.download.minecraft.net',
        replace_download_urls: false
    });

    const [originalSettings, setOriginalSettings] = useState<SettingsData>(settings);
    const [hasChanges, setHasChanges] = useState(false);

    useEffect(() => {
        const savedSettings = localStorage.getItem('modpack-settings');
        if (savedSettings) {
            const parsed = JSON.parse(savedSettings);
            setSettings(parsed);
            setOriginalSettings(parsed);
        }
    }, []);

    useEffect(() => {
        const changed =
            settings.download_server_base !== originalSettings.download_server_base ||
            settings.resources_url_base !== originalSettings.resources_url_base ||
            settings.replace_download_urls !== originalSettings.replace_download_urls;

        setHasChanges(changed);
    }, [settings, originalSettings]);

    const handleInputChange = (field: keyof SettingsData, value: string | boolean) => {
        setSettings(prev => ({...prev, [field]: value}));
    };

    const handleSave = () => {
        localStorage.setItem('modpack-settings', JSON.stringify(settings));
        setOriginalSettings(settings);
        setHasChanges(false);
        onSave(settings);
    };

    return (
        <div className="max-w-2xl mx-auto">
            <div className="bg-gray-800 rounded-lg border border-gray-700 p-8">
                <h2 className="text-2xl font-bold text-white mb-6">Application Settings</h2>

                <div className="space-y-6">
                    <div>
                        <label htmlFor="download_server_base" className="block text-sm font-medium text-gray-300 mb-2">
                            Download Server Base URL
                        </label>
                        <input
                            id="download_server_base"
                            type="text"
                            value={settings.download_server_base}
                            onChange={(e) => handleInputChange('download_server_base', e.target.value)}
                            className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                            placeholder="Enter download server base URL..."
                        />
                        <p className="mt-1 text-sm text-gray-500">Base URL for downloading Minecraft files</p>
                    </div>

                    <div>
                        <label htmlFor="resources_url_base" className="block text-sm font-medium text-gray-300 mb-2">
                            Resources URL Base
                        </label>
                        <input
                            id="resources_url_base"
                            type="text"
                            value={settings.resources_url_base}
                            onChange={(e) => handleInputChange('resources_url_base', e.target.value)}
                            className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                            placeholder="Enter resources URL base..."
                        />
                        <p className="mt-1 text-sm text-gray-500">Base URL for Minecraft resources and assets</p>
                    </div>

                    <div>
                        <label htmlFor="replace_download_urls" className="block text-sm font-medium text-gray-300 mb-2">
                            Replace Download URLs
                        </label>
                        <select
                            id="replace_download_urls"
                            value={settings.replace_download_urls.toString()}
                            onChange={(e) => handleInputChange('replace_download_urls', e.target.value === 'true')}
                            className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                        >
                            <option value="false">false</option>
                            <option value="true">true</option>
                        </select>
                        <p className="mt-1 text-sm text-gray-500">Whether to replace default download URLs with custom
                            ones</p>
                    </div>

                    <div className="pt-4">
                        <button
                            onClick={handleSave}
                            disabled={!hasChanges}
                            className={`flex items-center gap-2 px-6 py-3 rounded-md font-medium transition-all duration-200 ${
                                hasChanges
                                    ? 'bg-green-500 hover:bg-green-600 text-white cursor-pointer'
                                    : 'bg-gray-600 text-gray-400 cursor-not-allowed'
                            }`}
                        >
                            <Save size={20}/>
                            Save Settings
                        </button>
                        {hasChanges && (
                            <p className="mt-2 text-sm text-yellow-400">You have unsaved changes</p>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
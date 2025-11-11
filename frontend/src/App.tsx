import {useState} from 'react';
import ModpackSidebar from './components/ModpackSidebar';
import ModpackForm from './components/ModpackForm';
import ModpackDetails from './components/ModpackDetails';
import SettingsForm from './components/SettingsForm';

interface Modpack {
    id: string;
    name: string;
    version: string;
    loader: string;
    loaderVersion: string;
}

function App() {
    const [modpacks, setModpacks] = useState<Modpack[]>([
        {
            id: '1',
            name: 'Adventure Plus',
            version: '1.20.4',
            loader: 'forge',
            loaderVersion: '47.2.0'
        },
        {
            id: '2',
            name: 'Tech World',
            version: '1.20.1',
            loader: 'fabric',
            loaderVersion: '0.15.6'
        },
        {
            id: '3',
            name: 'Magic Realms',
            version: '1.19.4',
            loader: 'neoforge',
            loaderVersion: '2.4.15'
        }
    ]);

    const [selectedModpack, setSelectedModpack] = useState<string | null>(null);
    const [showForm, setShowForm] = useState(false);
    const [showSettings, setShowSettings] = useState(false);

    const handleNewModpack = () => {
        setShowForm(true);
        setSelectedModpack(null);
        setShowSettings(false);
    };

    const handleSelectModpack = (id: string) => {
        setSelectedModpack(id);
        setShowForm(false);
        setShowSettings(false);
    };

    const handleShowSettings = () => {
        setShowSettings(true);
        setShowForm(false);
        setSelectedModpack(null);
    };

    const handleModpackUpdate = (id: string, updatedData: Partial<Modpack>) => {
        setModpacks(prev => prev.map(modpack =>
            modpack.id === id ? {...modpack, ...updatedData} : modpack
        ));
    };

    const handleFormSubmit = (formData: { name: string; version: string; loader: string; loaderVersion: string }) => {
        const newModpack: Modpack = {
            id: Date.now().toString(),
            ...formData
        };

        setModpacks(prev => [...prev, newModpack]);
        setShowForm(false);
        setSelectedModpack(newModpack.id);
        setShowSettings(false);
    };

    const handleSettingsSave = (settings: any) => {
        console.log('Settings saved:', settings);
    };

    const selectedModpackData = modpacks.find(m => m.id === selectedModpack);

    return (
        <div className="min-h-screen bg-gray-900 flex">
            <ModpackSidebar
                modpacks={modpacks}
                selectedModpack={selectedModpack}
                onSelectModpack={handleSelectModpack}
                onNewModpack={handleNewModpack}
                showForm={showForm}
                onShowSettings={handleShowSettings}
                showSettings={showSettings}
            />

            <div className="flex-1 p-8">
                {showForm ? (
                    <ModpackForm onSubmit={handleFormSubmit}/>
                ) : showSettings ? (
                    <SettingsForm onSave={handleSettingsSave}/>
                ) : selectedModpackData ? (
                    <ModpackDetails
                        key={selectedModpack || 'none'}
                        modpack={selectedModpackData}
                        onUpdate={handleModpackUpdate}
                    />
                ) : (
                    <div className="flex items-center justify-center h-full">
                        <div className="text-center">
                            <h2 className="text-2xl font-bold text-gray-400 mb-4">
                                Welcome to Modpack Manager
                            </h2>
                            <p className="text-gray-500 mb-8">
                                Select a modpack from the sidebar or create a new one to get started.
                            </p>
                            <button
                                onClick={handleNewModpack}
                                className="bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded-md transition-colors duration-200"
                            >
                                Create Your First Modpack
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

export default App;
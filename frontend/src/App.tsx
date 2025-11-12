import {useState, useEffect} from 'react';
import LoginForm from './components/LoginForm';
import ModpackSidebar from './components/ModpackSidebar';
import ModpackForm from './components/ModpackForm';
import ModpackDetails from './components/ModpackDetails';
import SettingsForm from './components/SettingsForm';
import {useAuth} from './hooks/useAuth';
import {apiService} from './services/api';
import {ModpackResponse, ModpackBase} from './types/api';
import {useRef} from 'react';


function App() {
    const {isAuthenticated, loading: authLoading, error: authError, login, logout} = useAuth();
    const [modpacks, setModpacks] = useState<ModpackResponse[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [selectedModpack, setSelectedModpack] = useState<number | null>(null);
    const [showForm, setShowForm] = useState(false);
    const [showSettings, setShowSettings] = useState(false);
    const fetchingRef = useRef(false);

    // Set up unauthorized handler
    useEffect(() => {
        apiService.setUnauthorizedHandler(() => {
            logout();
        });
    }, [logout]);

    useEffect(() => {
        if (!isAuthenticated) return;

        if (fetchingRef.current) return;
        fetchingRef.current = true;

        loadModpacks();
    }, [isAuthenticated]);

    const loadModpacks = async () => {
        fetchingRef.current = false; // Reset flag to allow reloading
        try {
            setLoading(true);
            setError(null);
            const data = await apiService.getModpacks();
            setModpacks(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load modpacks');
        } finally {
            setLoading(false);
            fetchingRef.current = false;
        }
    };

    const handleNewModpack = () => {
        setShowForm(true);
        setSelectedModpack(null);
        setShowSettings(false);
    };

    const handleSelectModpack = (id: number) => {
        setSelectedModpack(id);
        setShowForm(false);
        setShowSettings(false);
    };

    const handleShowSettings = () => {
        setShowSettings(true);
        setShowForm(false);
        setSelectedModpack(null);
    };

    const handleModpackUpdate = (id: number, updatedData: Partial<ModpackResponse>) => {
        setModpacks(prev => prev.map(modpack =>
            modpack.id === id ? {...modpack, ...updatedData} : modpack
        ));
    };

    const handleModpackDelete = async (id: number) => {
        try {
            await apiService.deleteModpack(id);
            // Reload modpacks and clear selection
            setModpacks(prev => prev.filter(modpack => modpack.id !== id));
            if (selectedModpack === id) {
                setSelectedModpack(null);
            }
        } catch (err) {
            console.error('Failed to delete modpack:', err);
            // Reload from server in case of error
            await loadModpacks();
        }
    };

    const handleFormSubmit = async (formData: ModpackBase) => {
        try {
            const newModpack = await apiService.createModpack(formData);
            setModpacks(prev => [...prev, newModpack]);
            setShowForm(false);
            setSelectedModpack(newModpack.id);
            setShowSettings(false);
        } catch (err) {
            console.error('Failed to create modpack:', err);
        }
    };

    const handleSettingsSave = (settings: any) => {
        console.log('Settings saved:', settings);
    };

    // Show login form if not authenticated
    if (!isAuthenticated) {
        return (
            <LoginForm
                onLogin={login}
                loading={authLoading}
                error={authError}
            />
        );
    }

    const selectedModpackData = modpacks.find(m => m.id === selectedModpack);

    if (loading) {
        return (
            <div className="min-h-screen bg-gray-900 flex items-center justify-center">
                <div className="text-white text-xl">Loading...</div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="min-h-screen bg-gray-900 flex items-center justify-center">
                <div className="text-center">
                    <div className="text-red-400 text-xl mb-4">Error: {error}</div>
                    <button
                        onClick={loadModpacks}
                        className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
                    >
                        Retry
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-900 flex" style={{margin: '0 auto'}}>
            <ModpackSidebar
                modpacks={modpacks}
                selectedModpack={selectedModpack}
                onSelectModpack={handleSelectModpack}
                onNewModpack={handleNewModpack}
                showForm={showForm}
                onShowSettings={handleShowSettings}
                showSettings={showSettings}
                onLogout={logout}
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
                        onDelete={handleModpackDelete}
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
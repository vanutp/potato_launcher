import {useState, useEffect} from 'react';
import * as React from "react";

interface ModpackFormData {
    name: string;
    version: string;
    loader: string;
    loaderVersion: string;
}

interface ModpackFormProps {
    onSubmit: (data: ModpackFormData) => void;
}

const MINECRAFT_VERSIONS = [
    '1.20.4',
    '1.20.3',
    '1.20.2',
    '1.20.1',
    '1.19.4',
    '1.19.3',
    '1.19.2',
    '1.18.2',
    '1.18.1',
];

const LOADERS = [
    {id: 'forge', name: 'Forge'},
    {id: 'neoforge', name: 'NeoForge'},
    {id: 'fabric', name: 'Fabric'},
];

const LOADER_VERSIONS = {
    forge: ['47.2.0', '47.1.3', '47.1.0', '46.0.14', '45.2.0'],
    neoforge: ['2.4.15', '2.4.10', '2.3.5', '1.7.2', '1.5.8'],
    fabric: ['0.15.6', '0.15.3', '0.14.24', '0.14.21', '0.14.19'],
};

export default function ModpackForm({onSubmit}: ModpackFormProps) {
    const [formData, setFormData] = useState<ModpackFormData>({
        name: '',
        version: '',
        loader: '',
        loaderVersion: '',
    });

    const [errors, setErrors] = useState<{ [key: string]: string }>({});

    useEffect(() => {
        if (formData.version) {
            setFormData(prev => ({...prev, loader: '', loaderVersion: ''}));
        }
    }, [formData.version]);

    useEffect(() => {
        if (formData.loader) {
            setFormData(prev => ({...prev, loaderVersion: ''}));
        }
    }, [formData.loader]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        const newErrors: { [key: string]: string } = {};

        if (!formData.name.trim()) {
            newErrors.name = 'Name is required';
        }
        if (!formData.version) {
            newErrors.version = 'Version is required';
        }
        if (!formData.loader) {
            newErrors.loader = 'Loader is required';
        }
        if (!formData.loaderVersion) {
            newErrors.loaderVersion = 'Loader version is required';
        }

        setErrors(newErrors);

        if (Object.keys(newErrors).length === 0) {
            onSubmit(formData);
            setFormData({name: '', version: '', loader: '', loaderVersion: ''});
        }
    };

    const handleInputChange = (field: keyof ModpackFormData, value: string) => {
        setFormData(prev => ({...prev, [field]: value}));
        if (errors[field]) {
            setErrors(prev => ({...prev, [field]: ''}));
        }
    };

    return (
        <div className="max-w-2xl mx-auto">
            <div className="bg-gray-800 rounded-lg border border-gray-700 p-8">
                <h2 className="text-2xl font-bold text-white mb-6">Create New Modpack</h2>

                <form onSubmit={handleSubmit} className="space-y-6">
                    <div>
                        <label htmlFor="name" className="block text-sm font-medium text-gray-300 mb-2">
                            Modpack Name *
                        </label>
                        <input
                            id="name"
                            type="text"
                            value={formData.name}
                            onChange={(e) => handleInputChange('name', e.target.value)}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 transition-all duration-200 ${
                                errors.name
                                    ? 'border-red-500 focus:ring-red-500'
                                    : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                            placeholder="Enter modpack name..."
                        />
                        {errors.name && <p className="mt-1 text-sm text-red-400">{errors.name}</p>}
                    </div>

                    <div>
                        <label htmlFor="version" className="block text-sm font-medium text-gray-300 mb-2">
                            Minecraft Version *
                        </label>
                        <select
                            id="version"
                            value={formData.version}
                            onChange={(e) => handleInputChange('version', e.target.value)}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white focus:outline-none focus:ring-2 transition-all duration-200 ${
                                errors.version
                                    ? 'border-red-500 focus:ring-red-500'
                                    : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                        >
                            <option value="">Select Minecraft version...</option>
                            {MINECRAFT_VERSIONS.map((version) => (
                                <option key={version} value={version}>
                                    {version}
                                </option>
                            ))}
                        </select>
                        {errors.version && <p className="mt-1 text-sm text-red-400">{errors.version}</p>}
                    </div>

                    <div>
                        <label htmlFor="loader" className="block text-sm font-medium text-gray-300 mb-2">
                            Mod Loader *
                        </label>
                        <select
                            id="loader"
                            value={formData.loader}
                            onChange={(e) => handleInputChange('loader', e.target.value)}
                            disabled={!formData.version}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white focus:outline-none focus:ring-2 transition-all duration-200 ${
                                !formData.version
                                    ? 'opacity-50 cursor-not-allowed border-gray-600'
                                    : errors.loader
                                        ? 'border-red-500 focus:ring-red-500'
                                        : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                        >
                            <option value="">Select mod loader...</option>
                            {LOADERS.map((loader) => (
                                <option key={loader.id} value={loader.id}>
                                    {loader.name}
                                </option>
                            ))}
                        </select>
                        {errors.loader && <p className="mt-1 text-sm text-red-400">{errors.loader}</p>}
                        {!formData.version && (
                            <p className="mt-1 text-sm text-gray-500">Select a Minecraft version first</p>
                        )}
                    </div>

                    <div>
                        <label htmlFor="loaderVersion" className="block text-sm font-medium text-gray-300 mb-2">
                            Loader Version *
                        </label>
                        <select
                            id="loaderVersion"
                            value={formData.loaderVersion}
                            onChange={(e) => handleInputChange('loaderVersion', e.target.value)}
                            disabled={!formData.loader}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white focus:outline-none focus:ring-2 transition-all duration-200 ${
                                !formData.loader
                                    ? 'opacity-50 cursor-not-allowed border-gray-600'
                                    : errors.loaderVersion
                                        ? 'border-red-500 focus:ring-red-500'
                                        : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                        >
                            <option value="">Select loader version...</option>
                            {formData.loader && LOADER_VERSIONS[formData.loader as keyof typeof LOADER_VERSIONS]?.map((version) => (
                                <option key={version} value={version}>
                                    {version}
                                </option>
                            ))}
                        </select>
                        {errors.loaderVersion && <p className="mt-1 text-sm text-red-400">{errors.loaderVersion}</p>}
                        {!formData.loader && (
                            <p className="mt-1 text-sm text-gray-500">Select a mod loader first</p>
                        )}
                    </div>

                    <div className="pt-4">
                        <button
                            type="submit"
                            className="w-full bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded-md transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 focus:ring-offset-gray-800"
                        >
                            Create Modpack
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
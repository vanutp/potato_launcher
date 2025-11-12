import {useState, useEffect, useCallback} from 'react';
import * as React from "react";
import {apiService} from '../services/api';
import {ModpackBase, LoaderType} from '../types/api';

interface ModpackFormProps {
    onSubmit: (data: ModpackBase) => void;
}

export default function ModpackForm({onSubmit}: ModpackFormProps) {
    const [formData, setFormData] = useState<ModpackBase>({
        name: '',
        minecraft_version: '',
        loader: LoaderType.VANILLA,
        loader_version: '',
    });

    const [minecraftVersions, setMinecraftVersions] = useState<string[]>([]);
    const [availableLoaders, setAvailableLoaders] = useState<string[]>([]);
    const [loaderVersions, setLoaderVersions] = useState<string[]>([]);
    const [loading] = useState(false);
    const [errors, setErrors] = useState<{ [key: string]: string }>({});

    useEffect(() => {
        loadMinecraftVersions();
    }, []);

    const loadMinecraftVersions = async () => {
        try {
            const versions = await apiService.getMinecraftVersions();
            setMinecraftVersions(versions);
        } catch (err) {
            console.error('Failed to load Minecraft versions:', err);
        }
    };

    const loadLoaders = useCallback(async (version: string) => {
        if (!version) {
            setAvailableLoaders([]);
            return;
        }

        try {
            const loaders = await apiService.getLoadersForVersion(version);
            setAvailableLoaders(loaders);
        } catch (err) {
            console.error('Failed to load loaders:', err);
            setAvailableLoaders([]);
        }
    }, []);

    const loadLoaderVersions = useCallback(async (mcVersion: string, loader: string) => {
        if (!mcVersion || !loader) {
            setLoaderVersions([]);
            return;
        }

        try {
            const versions = await apiService.getLoaderVersions(mcVersion, loader);
            setLoaderVersions(versions);
        } catch (err) {
            console.error('Failed to load loader versions:', err);
            setLoaderVersions([]);
        }
    }, []);

    useEffect(() => {
        if (formData.minecraft_version) {
            loadLoaders(formData.minecraft_version);
            setFormData(prev => ({...prev, loader: LoaderType.VANILLA, loader_version: ''}));
        }
    }, [formData.minecraft_version, loadLoaders]);

    useEffect(() => {
        if (formData.minecraft_version && formData.loader) {
            loadLoaderVersions(formData.minecraft_version, formData.loader);
            setFormData(prev => ({...prev, loader_version: ''}));
        }
    }, [formData.minecraft_version, formData.loader, loadLoaderVersions]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        const newErrors: { [key: string]: string } = {};

        if (!formData.name.trim()) {
            newErrors.name = 'Name is required';
        }
        if (!formData.minecraft_version) {
            newErrors.minecraft_version = 'Minecraft version is required';
        }
        if (!formData.loader) {
            newErrors.loader = 'Loader is required';
        }
        if (!formData.loader_version) {
            newErrors.loader_version = 'Loader version is required';
        }

        setErrors(newErrors);

        if (Object.keys(newErrors).length === 0) {
            onSubmit(formData);
            setFormData({name: '', minecraft_version: '', loader: LoaderType.VANILLA, loader_version: ''});
        }
    };

    const handleInputChange = (field: keyof ModpackBase, value: string | LoaderType) => {
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
                        <label htmlFor="minecraft_version" className="block text-sm font-medium text-gray-300 mb-2">
                            Minecraft Version *
                        </label>
                        <select
                            id="minecraft_version"
                            value={formData.minecraft_version}
                            onChange={(e) => handleInputChange('minecraft_version', e.target.value)}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white focus:outline-none focus:ring-2 transition-all duration-200 ${
                                errors.minecraft_version
                                    ? 'border-red-500 focus:ring-red-500'
                                    : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                        >
                            <option value="">Select Minecraft version...</option>
                            {minecraftVersions.map((version) => (
                                <option key={version} value={version}>
                                    {version}
                                </option>
                            ))}
                        </select>
                        {errors.minecraft_version &&
                            <p className="mt-1 text-sm text-red-400">{errors.minecraft_version}</p>}
                    </div>

                    <div>
                        <label htmlFor="loader" className="block text-sm font-medium text-gray-300 mb-2">
                            Mod Loader *
                        </label>
                        <select
                            id="loader"
                            value={formData.loader}
                            onChange={(e) => handleInputChange('loader', e.target.value as LoaderType)}
                            disabled={!formData.minecraft_version || availableLoaders.length === 0}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white focus:outline-none focus:ring-2 transition-all duration-200 ${
                                !formData.minecraft_version || availableLoaders.length === 0
                                    ? 'opacity-50 cursor-not-allowed border-gray-600'
                                    : errors.loader
                                        ? 'border-red-500 focus:ring-red-500'
                                        : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                        >
                            <option value="">Select mod loader...</option>
                            {availableLoaders.map((loader) => (
                                <option key={loader} value={loader}>
                                    {loader.charAt(0).toUpperCase() + loader.slice(1)}
                                </option>
                            ))}
                        </select>
                        {errors.loader && <p className="mt-1 text-sm text-red-400">{errors.loader}</p>}
                        {!formData.minecraft_version && (
                            <p className="mt-1 text-sm text-gray-500">Select a Minecraft version first</p>
                        )}
                        {formData.minecraft_version && availableLoaders.length === 0 && (
                            <p className="mt-1 text-sm text-yellow-400">No loaders available for this version</p>
                        )}
                    </div>

                    <div>
                        <label htmlFor="loader_version" className="block text-sm font-medium text-gray-300 mb-2">
                            Loader Version *
                        </label>
                        <select
                            id="loader_version"
                            value={formData.loader_version}
                            onChange={(e) => handleInputChange('loader_version', e.target.value)}
                            disabled={!formData.loader || loaderVersions.length === 0}
                            className={`w-full px-4 py-3 bg-gray-700 border rounded-md text-white focus:outline-none focus:ring-2 transition-all duration-200 ${
                                !formData.loader || loaderVersions.length === 0
                                    ? 'opacity-50 cursor-not-allowed border-gray-600'
                                    : errors.loader_version
                                        ? 'border-red-500 focus:ring-red-500'
                                        : 'border-gray-600 focus:ring-green-500 focus:border-green-500'
                            }`}
                        >
                            <option value="">Select loader version...</option>
                            {loaderVersions.map((version) => (
                                <option key={version} value={version}>
                                    {version}
                                </option>
                            ))}
                        </select>
                        {errors.loader_version && <p className="mt-1 text-sm text-red-400">{errors.loader_version}</p>}
                        {!formData.loader && (
                            <p className="mt-1 text-sm text-gray-500">Select a mod loader first</p>
                        )}
                        {formData.loader && loaderVersions.length === 0 && (
                            <p className="mt-1 text-sm text-yellow-400">No versions available for this loader</p>
                        )}
                    </div>

                    <div className="pt-4">
                        <button
                            type="submit"
                            disabled={loading}
                            className="w-full bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded-md transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 focus:ring-offset-gray-800"
                        >
                            {loading ? 'Creating...' : 'Create Modpack'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
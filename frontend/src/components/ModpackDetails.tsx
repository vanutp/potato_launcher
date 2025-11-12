import {useState, useEffect, useCallback} from 'react';
import {CreditCard as Edit, Save, Upload, X, Trash2} from 'lucide-react';
import DeleteConfirmModal from './DeleteConfirmModal';
import { ModpackResponse, LoaderType } from '../types/api';
import { apiService } from '../services/api';

interface ModpackDetailsProps {
    modpack: ModpackResponse;
    onUpdate: (id: number, updatedData: Partial<ModpackResponse>) => void;
    onDelete: (id: number) => void;
}

export default function ModpackDetails({ modpack, onUpdate, onDelete }: ModpackDetailsProps) {
    const [isEditing, setIsEditing] = useState(false);
    const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
    const [editData, setEditData] = useState({
        name: modpack.name,
        minecraft_version: modpack.minecraft_version,
        loader: modpack.loader,
        loader_version: modpack.loader_version,
    });

    const [minecraftVersions, setMinecraftVersions] = useState<string[]>([]);
    const [availableLoaders, setAvailableLoaders] = useState<string[]>([]);
    const [loaderVersions, setLoaderVersions] = useState<string[]>([]);
    const [loadingVersions, setLoadingVersions] = useState(false);
    const [loadingLoaders, setLoadingLoaders] = useState(false);
    const [loadingLoaderVersions, setLoadingLoaderVersions] = useState(false);
    const [dragActive, setDragActive] = useState(false);
    const [uploadedFiles, setUploadedFiles] = useState<FileList | null>(null);

    const loadMinecraftVersions = async () => {
        setLoadingVersions(true);
        try {
            const versions = await apiService.getMinecraftVersions();
            setMinecraftVersions(versions);
        } catch (err) {
            console.error('Failed to load Minecraft versions:', err);
        } finally {
            setLoadingVersions(false);
        }
    };

    const loadLoaders = useCallback(async (version: string) => {
        if (!version) {
            setAvailableLoaders([]);
            return;
        }

        setLoadingLoaders(true);
        try {
            const loaders = await apiService.getLoadersForVersion(version);
            setAvailableLoaders(loaders);
        } catch (err) {
            console.error('Failed to load loaders:', err);
            setAvailableLoaders([]);
        } finally {
            setLoadingLoaders(false);
        }
    }, []);

    const loadLoaderVersions = useCallback(async (mcVersion: string, loader: string) => {
        if (!mcVersion || !loader) {
            setLoaderVersions([]);
            return;
        }

        setLoadingLoaderVersions(true);
        try {
            const versions = await apiService.getLoaderVersions(mcVersion, loader);
            setLoaderVersions(versions);
        } catch (err) {
            console.error('Failed to load loader versions:', err);
            setLoaderVersions([]);
        } finally {
            setLoadingLoaderVersions(false);
        }
    }, []);

    useEffect(() => {
        // Reset state when modpack changes
        handleCancel();
    }, [modpack.id]);

    useEffect(() => {
        if (isEditing && editData.minecraft_version) {
            loadLoaders(editData.minecraft_version);
        }
    }, [editData.minecraft_version, loadLoaders, isEditing]);

    useEffect(() => {
        if (isEditing && editData.minecraft_version && editData.loader) {
            loadLoaderVersions(editData.minecraft_version, editData.loader);
        }
    }, [editData.minecraft_version, editData.loader, loadLoaderVersions, isEditing]);

    const handleEdit = async () => {
        setIsEditing(true);
        setEditData({
            name: modpack.name,
            minecraft_version: modpack.minecraft_version,
            loader: modpack.loader,
            loader_version: modpack.loader_version,
        });

        // Load data when starting to edit
        await loadMinecraftVersions();
    };

    const handleCancel = () => {
        setIsEditing(false);
        setShowDeleteConfirm(false);
        setUploadedFiles(null);
        // Clear loaded data
        setMinecraftVersions([]);
        setAvailableLoaders([]);
        setLoaderVersions([]);
        setEditData({
            name: modpack.name,
            minecraft_version: modpack.minecraft_version,
            loader: modpack.loader,
            loader_version: modpack.loader_version,
        });
    };

    const handleUpdate = () => {
        onUpdate(modpack.id, editData);
        setIsEditing(false);
        setShowDeleteConfirm(false);
        setUploadedFiles(null);
    };

    const handleInputChange = (field: keyof typeof editData, value: string | LoaderType) => {
        setEditData(prev => ({...prev, [field]: value}));
    };

    const handleDelete = () => {
        onDelete(modpack.id);
    };

    const handleDrag = (e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.type === 'dragenter' || e.type === 'dragover') {
            setDragActive(true);
        } else if (e.type === 'dragleave') {
            setDragActive(false);
        }
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setDragActive(false);

        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
            setUploadedFiles(e.dataTransfer.files);
        }
    };

    const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files[0]) {
            setUploadedFiles(e.target.files);
        }
    };

    return (
        <div className="max-w-2xl mx-auto">
            <div className="bg-gray-800 rounded-lg border border-gray-700 p-8">
                <div className="flex items-center justify-between mb-6">
                    <h2 className="text-2xl font-bold text-white">
                        {isEditing ? 'Edit Modpack' : modpack.name}
                    </h2>
                    {!isEditing && (
                        <div className="flex items-center gap-3">
                            <button
                                onClick={handleEdit}
                                className="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-md transition-colors duration-200"
                            >
                                <Edit size={16}/>
                                Update modpack
                            </button>
                            {!showDeleteConfirm ? (
                                <button
                                    onClick={() => setShowDeleteConfirm(true)}
                                    className="flex items-center gap-2 px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-md transition-colors duration-200"
                                >
                                    <Trash2 size={16}/>
                                    Delete
                                </button>
                            ) : (
                                <div className="flex items-center gap-2">
                                    <span className="text-red-400 text-sm font-medium">Delete this modpack?</span>
                                    <button
                                        onClick={handleDelete}
                                        className="px-3 py-1 bg-red-600 hover:bg-red-700 text-white text-sm rounded transition-colors duration-200"
                                    >
                                        Yes
                                    </button>
                                    <button
                                        onClick={() => setShowDeleteConfirm(false)}
                                        className="px-3 py-1 bg-gray-600 hover:bg-gray-700 text-white text-sm rounded transition-colors duration-200"
                                    >
                                        No
                                    </button>
                                </div>
                            )}
                        </div>
                    )}
                </div>

                <div className="space-y-6">
                    {isEditing ? (
                        <>
                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Modpack Name *
                                </label>
                                <input
                                    type="text"
                                    value={editData.name}
                                    onChange={(e) => handleInputChange('name', e.target.value)}
                                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Minecraft Version *
                                </label>
                                {loadingVersions ? (
                                    <div className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-gray-400 flex items-center">
                                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-green-500 mr-2"></div>
                                        Loading versions...
                                    </div>
                                ) : (
                                    <select
                                        value={editData.minecraft_version}
                                        onChange={(e) => handleInputChange('minecraft_version', e.target.value)}
                                        className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                                    >
                                        {minecraftVersions.map((version) => (
                                            <option key={version} value={version}>
                                                {version}
                                            </option>
                                        ))}
                                    </select>
                                )}
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Mod Loader *
                                </label>
                                {loadingLoaders ? (
                                    <div className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-gray-400 flex items-center">
                                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-green-500 mr-2"></div>
                                        Loading loaders...
                                    </div>
                                ) : (
                                    <select
                                        value={editData.loader}
                                        onChange={(e) => handleInputChange('loader', e.target.value as LoaderType)}
                                        disabled={!editData.minecraft_version || availableLoaders.length === 0}
                                        className={`w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200 ${
                                            !editData.minecraft_version || availableLoaders.length === 0
                                                ? 'opacity-50 cursor-not-allowed'
                                                : ''
                                        }`}
                                    >
                                        {availableLoaders.map((loader) => (
                                            <option key={loader} value={loader}>
                                                {loader.charAt(0).toUpperCase() + loader.slice(1)}
                                            </option>
                                        ))}
                                    </select>
                                )}
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Loader Version *
                                </label>
                                {loadingLoaderVersions ? (
                                    <div className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-gray-400 flex items-center">
                                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-green-500 mr-2"></div>
                                        Loading versions...
                                    </div>
                                ) : (
                                    <select
                                        value={editData.loader_version}
                                        onChange={(e) => handleInputChange('loader_version', e.target.value)}
                                        disabled={!editData.loader || loaderVersions.length === 0}
                                        className={`w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200 ${
                                            !editData.loader || loaderVersions.length === 0
                                                ? 'opacity-50 cursor-not-allowed'
                                                : ''
                                        }`}
                                    >
                                        {loaderVersions.map((version) => (
                                            <option key={version} value={version}>
                                                {version}
                                            </option>
                                        ))}
                                    </select>
                                )}
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Upload Files (Optional)
                                </label>
                                <div
                                    className={`relative border-2 border-dashed rounded-lg p-8 text-center transition-all duration-200 ${
                                        dragActive
                                            ? 'border-green-500 bg-green-500/10'
                                            : 'border-gray-600 hover:border-gray-500'
                                    }`}
                                    onDragEnter={handleDrag}
                                    onDragLeave={handleDrag}
                                    onDragOver={handleDrag}
                                    onDrop={handleDrop}
                                >
                                    <input
                                        type="file"
                                        multiple
                                        onChange={handleFileInput}
                                        className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                                        webkitdirectory=""
                                    />
                                    <Upload className="mx-auto h-12 w-12 text-gray-400 mb-4"/>
                                    <p className="text-gray-300 mb-2">
                                        Drag and drop modpack folder here, or click to browse
                                    </p>
                                    <p className="text-sm text-gray-500">
                                        Upload your modpack files and folders
                                    </p>
                                    {uploadedFiles && (
                                        <div className="mt-4 p-3 bg-gray-700 rounded-md">
                                            <p className="text-green-400 text-sm">
                                                {uploadedFiles.length} file(s) selected
                                            </p>
                                        </div>
                                    )}
                                </div>
                            </div>

                            <div className="flex gap-3 pt-4">
                                <button
                                    onClick={handleUpdate}
                                    className="flex items-center gap-2 px-6 py-3 bg-green-500 hover:bg-green-600 text-white font-medium rounded-md transition-colors duration-200"
                                >
                                    <Save size={16}/>
                                    Update
                                </button>
                                <button
                                    onClick={handleCancel}
                                    className="flex items-center gap-2 px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-medium rounded-md transition-colors duration-200"
                                >
                                    <X size={16}/>
                                    Cancel
                                </button>
                            </div>
                        </>
                    ) : (
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-400 mb-1">
                                    Minecraft Version
                                </label>
                                <p className="text-white bg-gray-700 px-4 py-2 rounded-md">
                                    {modpack.minecraft_version}
                                </p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-400 mb-1">
                                    Mod Loader
                                </label>
                                <p className="text-white bg-gray-700 px-4 py-2 rounded-md capitalize">
                                    {modpack.loader}
                                </p>
                            </div>
                            <div className="col-span-2">
                                <label className="block text-sm font-medium text-gray-400 mb-1">
                                    Loader Version
                                </label>
                                <p className="text-white bg-gray-700 px-4 py-2 rounded-md">
                                    {modpack.loader_version}
                                </p>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            <DeleteConfirmModal
                isOpen={showDeleteConfirm}
                modpackName={modpack.name}
                onConfirm={handleDelete}
                onCancel={() => setShowDeleteConfirm(false)}
            />
        </div>
    );
}
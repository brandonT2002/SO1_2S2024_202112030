#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/jiffies.h>
#include <linux/uaccess.h>
#include <linux/fs.h>
#include <linux/sched/signal.h>
#include <linux/slab.h>
#include <linux/binfmts.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Brandon Tejaxún");
MODULE_DESCRIPTION("Modulo para leer informacion de memoria y CPU en JSON");
MODULE_VERSION("1.0");

#define PROC_NAME "sysinfo_202112030"
#define MAX_CMDLINE_LENGTH 256

static char *get_process_cmdline(struct task_struct *task) {
    struct mm_struct *mm;
    char *cmdline, *p;
    unsigned long arg_start, arg_end;
    int len, i;

    cmdline = kmalloc(MAX_CMDLINE_LENGTH, GFP_KERNEL);
    if (!cmdline)
        return NULL;

    mm = get_task_mm(task);
    if (!mm) {
        kfree(cmdline);
        return NULL;
    }

    down_read(&mm->mmap_lock);
    arg_start = mm->arg_start;
    arg_end = mm->arg_end;
    up_read(&mm->mmap_lock);

    len = arg_end - arg_start;
    if (len > MAX_CMDLINE_LENGTH - 1)
        len = MAX_CMDLINE_LENGTH - 1;

    if (access_process_vm(task, arg_start, cmdline, len, 0) != len) {
        mmput(mm);
        kfree(cmdline);
        return NULL;
    }

    cmdline[len] = '\0';
    p = cmdline;
    for (i = 0; i < len; i++)
        if (p[i] == '\0')
            p[i] = ' ';

    mmput(mm);
    return cmdline;
}

static int sysinfo_show(struct seq_file *m, void *v) {
    struct sysinfo si;
    struct task_struct *task;
    unsigned long totalram, freeram, usedram;
    unsigned long total_jiffies = jiffies;
    unsigned long total_cpu_time = 0;
    int first_process = 1;

    // Obtener la información de memoria
    si_meminfo(&si);
    totalram = si.totalram << (PAGE_SHIFT - 10); // totalram en KB
    freeram = si.freeram << (PAGE_SHIFT - 10);   // freeram en KB
    usedram = totalram - freeram;                // usedram en KB

    // Mostrar la información de memoria en JSON
    seq_printf(m, "{\n");
    seq_printf(m, "\"Memory\": {\n");
    seq_printf(m, "\t\"TotalRAM\": %lu KB,\n", totalram);
    seq_printf(m, "\t\"FreeRAM\": %lu KB,\n", freeram);
    seq_printf(m, "\t\"UsedRAM\": %lu KB\n", usedram);
    seq_printf(m, "},\n");

    // Mostrar la información de procesos en JSON
    seq_printf(m, "\"Processes\": [\n");

    for_each_process(task) {
        if (strcmp(task->comm, "containerd-shim") == 0) {
            unsigned long vsz = 0;
            unsigned long rss = 0;
            unsigned long mem_usage = 0;
            unsigned long cpu_usage = 0;
            char *cmdline = NULL;

            if (task->mm) {
                // Obtener el tamaño de memoria virtual y física
                vsz = task->mm->total_vm << (PAGE_SHIFT - 10); // vsz en KB
                rss = get_mm_rss(task->mm) << (PAGE_SHIFT - 10); // rss en KB
                mem_usage = (rss * 100) / totalram;
            }

            // Obtener el tiempo total de CPU para este proceso
            unsigned long total_time = task->utime + task->stime;
            total_cpu_time += total_time;

            // Obtener el uso de CPU como porcentaje
            cpu_usage = (total_time * 100) / total_jiffies;

            cmdline = get_process_cmdline(task);

            if (!first_process) {
                seq_printf(m, ",\n");
            } else {
                first_process = 0;
            }

            seq_printf(m, "\t{\n");
            seq_printf(m, "\t\t\"PID\": %d,\n", task->pid);
            seq_printf(m, "\t\t\"Name\": \"%s\",\n", task->comm);
            seq_printf(m, "\t\t\"Cmdline\": \"%s\",\n", cmdline ? cmdline : "N/A");
            seq_printf(m, "\t\t\"MemoryUsage\": %lu KB,\n", mem_usage / 100, mem_usage % 100);
            seq_printf(m, "\t\t\"CPUUsage\": %lu.%02lu%%\n", cpu_usage, cpu_usage % 100);
            seq_printf(m, "\t}");

            if (cmdline) {
                kfree(cmdline);
            }
        }
    }

    seq_printf(m, "\n]\n}\n");
    return 0;
}


static int sysinfo_open(struct inode *inode, struct file *file) {
    return single_open(file, sysinfo_show, NULL);
}

static const struct proc_ops sysinfo_ops = {
    .proc_open = sysinfo_open,
    .proc_read = seq_read,
};

static int __init sysinfo_init(void) {
    proc_create(PROC_NAME, 0, NULL, &sysinfo_ops);
    printk(KERN_INFO "sysinfo_202112030 loaded\n");
    return 0;
}

static void __exit sysinfo_exit(void) {
    remove_proc_entry(PROC_NAME, NULL);
    printk(KERN_INFO "sysinfo_202112030 unloaded\n");
}

module_init(sysinfo_init);
module_exit(sysinfo_exit);
